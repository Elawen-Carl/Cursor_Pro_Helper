use crate::utils;
use anyhow::{Context, Result};
use rand::Rng;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

pub struct Patcher {
    js_path: PathBuf,
    data: String,
}

impl Patcher {
    pub fn new(js_path: Option<&str>) -> Result<Self> {
        let path = if let Some(p) = js_path {
            PathBuf::from(p)
        } else {
            Self::get_default_js_path().context("无法获取默认的 main.js 路径")?
        };

        if !path.exists() {
            return Err(anyhow::anyhow!("main.js not found"));
        }

        let data = fs::read_to_string(&path).context("读取 main.js 失败")?;

        Ok(Self {
            js_path: path,
            data,
        })
    }

    /// 设置文件的权限
    async fn set_file_permissions(file_path: &PathBuf, readonly: bool) -> Result<()> {
        let mut perms = tokio::fs::metadata(file_path).await?.permissions();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if readonly {
                perms.set_mode(0o444); // r--r--r--
            } else {
                perms.set_mode(0o644); // rw-r--r--
            }
        }

        #[cfg(windows)]
        {
            perms.set_readonly(readonly);
        }

        tokio::fs::set_permissions(file_path, perms)
            .await
            .context(format!("设置文件权限失败: {:?}", file_path))?;

        Ok(())
    }

    /// 删除文件，如果文件存在且为只读，先解除只读属性
    async fn remove_file_if_exists(file_path: &PathBuf) -> Result<()> {
        if file_path.exists() {
            // 先解除只读属性
            Self::set_file_permissions(file_path, false).await?;
            // 然后删除文件
            tokio::fs::remove_file(file_path)
                .await
                .context(format!("删除文件失败: {:?}", file_path))?;
        }
        Ok(())
    }

    /// 备份 main.js 文件
    pub async fn backup(&self) -> Result<()> {
        info!("开始备份 main.js 文件");
        let backup_path = self.js_path.with_extension("js.bak");

        // 确保备份文件的父目录存在
        if let Some(parent) = backup_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context(format!("创建备份目录失败: {:?}", parent))?;
        }

        // 删除旧的备份文件（如果存在）
        Self::remove_file_if_exists(&backup_path).await?;

        // 执行备份
        tokio::fs::copy(&self.js_path, &backup_path)
            .await
            .context(format!(
                "备份 main.js 失败: 从 {:?} 复制到 {:?}",
                self.js_path, backup_path
            ))?;

        // 设置备份文件为只读
        Self::set_file_permissions(&backup_path, true).await?;

        info!("main.js 备份完成");
        Ok(())
    }

    /// 还原 main.js 文件
    pub async fn restore(&self) -> Result<()> {
        info!("开始还原 main.js 文件");
        let backup_path = self.js_path.with_extension("js.bak");

        if !backup_path.exists() {
            let err_msg = format!("备份文件不存在: {:?}", backup_path);
            error!("{}", err_msg);
            return Err(anyhow::anyhow!(err_msg));
        }

        // 删除当前文件
        Self::remove_file_if_exists(&self.js_path).await?;

        // 执行还原
        tokio::fs::copy(&backup_path, &self.js_path)
            .await
            .context(format!(
                "还原 main.js 失败: 从 {:?} 复制到 {:?}",
                backup_path, self.js_path
            ))?;

        info!("main.js 还原完成");
        Ok(())
    }

    pub fn get_js_path(&self) -> &PathBuf {
        &self.js_path
    }

    fn get_default_js_path() -> Result<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
                let path = PathBuf::from(local_app_data)
                    .join("Programs")
                    .join("cursor")
                    .join("resources")
                    .join("app")
                    .join("out")
                    .join("main.js");
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let path = PathBuf::from("/Applications/Cursor.app/Contents/Resources/app/out/main.js");
            if path.exists() {
                return Ok(path);
            }
        }

        #[cfg(target_os = "linux")]
        {
            let paths = vec![
                "/opt/Cursor/resources/app/out/main.js",
                "/usr/share/cursor/resources/app/out/main.js",
            ];
            for p in paths {
                let path = PathBuf::from(p);
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        Err(anyhow::anyhow!("Could not find default main.js path"))
    }

    pub fn generate_mac_address() -> String {
        let mut rng = rand::thread_rng();
        let invalid_macs = vec![
            "00:00:00:00:00:00",
            "ff:ff:ff:ff:ff:ff",
            "ac:de:48:00:11:22",
        ];

        loop {
            let mac: String = (0..6)
                .map(|_| format!("{:02X}", rng.gen_range(0..=255)))
                .collect::<Vec<String>>()
                .join(":");

            if !invalid_macs.contains(&mac.as_str()) {
                return mac;
            }
        }
    }

    pub fn patch(
        &mut self,
        machine_id: Option<String>,
        mac_addr: Option<String>,
        sqm_id: Option<String>,
        dev_device_id: Option<String>,
    ) -> Result<()> {
        // 备份原始文件
        let backup_path = self.js_path.with_extension("js.bak");
        if !backup_path.exists() {
            fs::copy(&self.js_path, &backup_path).context("备份 main.js 失败")?;
        }

        // 解除文件的只读属性
        let mut perms = fs::metadata(&self.js_path)
            .context("获取文件权限失败")?
            .permissions();
        perms.set_readonly(false);
        fs::set_permissions(&self.js_path, perms).context("解除文件只读属性失败")?;

        let mut data = self.data.clone();

        // Patch machine ID
        let machine_id = machine_id.unwrap_or_else(|| utils::generate_uuid());
        info!("Patching machine ID: {}", machine_id);
        data = self.replace_in_data(
            &data,
            r"=.{0,50}timeout.{0,10}5e3.*?,",
            &format!("=/*csp1*/\"{}\"/*1csp*/,", machine_id),
            r"=/\*csp1\*/.*?/\*1csp\*/,",
        )?;

        // Patch MAC address
        let mac = mac_addr.unwrap_or_else(Self::generate_mac_address);
        info!("Patching MAC address: {}", mac);
        data = self.replace_in_data(
            &data,
            r"(function .{0,50}\{).{0,300}Unable to retrieve mac address.*?(\})",
            &format!(r#"$1 return/*csp2*/"{}"/*2csp*/; $2"#, mac),
            r"()return/\*csp2\*/.*?/\*2csp\*/;()",
        )?;

        // Patch SQM ID
        let sqm = sqm_id.unwrap_or_else(String::new);
        info!("Patching SQM ID: {}", sqm);
        data = self.replace_in_data(
            &data,
            r#"return.{0,50}\.GetStringRegKey.*?HKEY_LOCAL_MACHINE.*?MachineId.*?\|\|.*?"""#,
            &format!("return/*csp3*/\"{}\"/*3csp*/", sqm),
            r"return/\*csp3\*/.*?/\*3csp\*/",
        )?;

        // Patch device ID
        let dev_id = dev_device_id.unwrap_or_else(|| utils::generate_uuid());
        info!("Patching device ID: {}", dev_id);
        data = self.replace_in_data(
            &data,
            r"return.{0,50}vscode/deviceid.*?getDeviceId\(\)",
            &format!("return/*csp4*/\"{}\"/*4csp*/", dev_id),
            r"return/\*csp4\*/.*?/\*4csp\*/",
        )?;

        // 保存修改
        fs::write(&self.js_path, &data).context("保存修改失败")?;

        // 更新内部数据
        self.data = data;

        Ok(())
    }

    fn replace_in_data(
        &self,
        data: &str,
        pattern: &str,
        replacement: &str,
        probe: &str,
    ) -> Result<String> {
        let regex = Regex::new(pattern).context("创建正则表达式失败")?;
        let probe_regex = Regex::new(probe).context("创建探测正则表达式失败")?;

        let count = regex.find_iter(data).count();
        let patched_count = probe_regex.find_iter(data).count();

        if count == 0 {
            if patched_count > 0 {
                info!(
                    "Found {} pattern{} already patched, will overwrite",
                    patched_count,
                    if patched_count == 1 { "" } else { "s" }
                );
            } else {
                info!("Warning: Pattern not found, SKIPPED!");
                return Ok(data.to_string());
            }
        }

        let mut result = data.to_string();
        let mut replaced1_count = 0;
        let mut replaced2_count = 0;

        // 先处理已经打过补丁的内容
        if patched_count > 0 {
            let new_result = probe_regex.replace_all(&result, replacement);
            replaced1_count = patched_count - probe_regex.find_iter(&new_result).count();
            result = new_result.into_owned();
        }

        // 处理新的匹配
        if count > 0 {
            let new_result = regex.replace_all(&result, replacement);
            replaced2_count = count - regex.find_iter(&new_result).count();
            result = new_result.into_owned();
        }

        let replaced_count = replaced1_count + replaced2_count;
        if replaced_count != count + patched_count {
            info!(
                "Warning: Patched {}/{}, failed {}",
                replaced_count,
                count + patched_count,
                count + patched_count - replaced_count
            );
        } else {
            info!(
                "Patched {} pattern{}",
                replaced_count,
                if replaced_count == 1 { "" } else { "s" }
            );
        }

        Ok(result)
    }

    pub fn restore_original(&self) -> Result<(), std::io::Error> {
        let backup_path = self.js_path.with_extension("js.bak");
        if backup_path.exists() {
            fs::copy(&backup_path, &self.js_path)?;
        }
        Ok(())
    }

    pub fn get_current_ids(&self) -> Result<(String, String, String, String)> {
        // 使用与 Python 版本相同的正则表达式格式
        let machine_id_regex = Regex::new(r#"/\*csp1\*/"([^"]*)"/\*1csp\*/"#)
            .context("创建 machine ID 正则表达式失败")?;
        let mac_regex = Regex::new(r#"/\*csp2\*/"([^"]*)"/\*2csp\*/"#)
            .context("创建 MAC address 正则表达式失败")?;
        let sqm_regex =
            Regex::new(r#"/\*csp3\*/"([^"]*)"/\*3csp\*/"#).context("创建 SQM ID 正则表达式失败")?;
        let device_id_regex = Regex::new(r#"/\*csp4\*/"([^"]*)"/\*4csp\*/"#)
            .context("创建 device ID 正则表达式失败")?;

        let machine_id = match machine_id_regex.captures(&self.data) {
            Some(cap) => {
                info!("Found machine_id match: {:?}", cap.get(1));
                cap.get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default()
            }
            None => {
                info!("No machine_id match found");
                String::default()
            }
        };

        let mac_machine_id = match mac_regex.captures(&self.data) {
            Some(cap) => {
                info!("Found mac match: {:?}", cap.get(1));
                cap.get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default()
            }
            None => {
                info!("No mac match found");
                String::default()
            }
        };

        let sqm_id = match sqm_regex.captures(&self.data) {
            Some(cap) => {
                info!("Found sqm match: {:?}", cap.get(1));
                cap.get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default()
            }
            None => {
                info!("No sqm match found");
                String::default()
            }
        };

        let dev_device_id = match device_id_regex.captures(&self.data) {
            Some(cap) => {
                info!("Found device match: {:?}", cap.get(1));
                cap.get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default()
            }
            None => {
                info!("No device match found");
                String::default()
            }
        };

        Ok((machine_id, mac_machine_id, dev_device_id, sqm_id))
    }
}
