//! 机器 ID 服务模块
//!
//! 提供机器 ID 的管理功能，包括：
//! - 获取当前机器 ID
//! - 生成新的机器 ID
//! - 更新机器 ID
//! - 备份和还原

use crate::auth_manager;
use crate::events::{NoopProgressEmitter, ProgressEmitter};
use crate::utils;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;
use tokio::time::sleep;
use tokio::time::Duration;
use tracing::{error, info, warn};

#[cfg(windows)]
use sysinfo::{ProcessExt, System, SystemExt};
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// 机器服务
pub struct MachineService {
    /// 配置文件路径
    config_path: PathBuf,
    /// 备份文件路径
    backup_path: PathBuf,
    /// 进度事件发送器
    progress_emitter: Box<dyn ProgressEmitter>,
}

impl MachineService {
    /// 创建新的机器服务实例
    pub async fn new() -> Result<Self> {
        let config_path = Self::get_storage_path(None)?;
        let backup_path = config_path.with_extension("json.bak");

        // 确保父目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("创建配置目录失败")?;
        }

        let instance = Self {
            config_path,
            backup_path,
            progress_emitter: Box::new(NoopProgressEmitter),
        };

        // 确保配置文件存在
        if !instance.config_path.exists() {
            info!("配置文件不存在，正在创建默认配置");
            let default_config = utils::generate_default_machine_config();
            instance.write_config(&default_config).await?;
            info!("默认配置创建成功");
        }

        Ok(instance)
    }

    /// 使用进度发送器创建实例
    pub async fn with_progress(progress_emitter: Box<dyn ProgressEmitter>) -> Result<Self> {
        let config_path = Self::get_storage_path(None)?;
        let backup_path = config_path.with_extension("json.bak");

        // 确保父目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("创建配置目录失败")?;
        }

        let instance = Self {
            config_path,
            backup_path,
            progress_emitter,
        };

        // 确保配置文件存在
        if !instance.config_path.exists() {
            info!("配置文件不存在，正在创建默认配置");
            let default_config = utils::generate_default_machine_config();
            instance.write_config(&default_config).await?;
            info!("默认配置创建成功");
        }

        Ok(instance)
    }

    /// 获取配置文件路径
    fn get_storage_path(root_dir: Option<PathBuf>) -> Result<PathBuf> {
        let home_dir = root_dir.unwrap_or_else(|| dirs::home_dir().expect("无法获取用户主目录"));

        let path = match std::env::consts::OS {
            "windows" => home_dir
                .join("AppData")
                .join("Roaming")
                .join("Cursor")
                .join("User")
                .join("globalStorage")
                .join("storage.json"),
            "macos" => home_dir
                .join("Library")
                .join("Application Support")
                .join("Cursor")
                .join("User")
                .join("globalStorage")
                .join("storage.json"),
            "linux" => home_dir
                .join(".config")
                .join("Cursor")
                .join("User")
                .join("globalStorage")
                .join("storage.json"),
            _ => return Err(anyhow::anyhow!("不支持的操作系统")),
        };

        Ok(path)
    }

    /// 发送进度事件
    fn emit_progress(&self, message: &str) {
        self.progress_emitter.emit_progress(message);
    }

    /// 读取配置文件
    async fn read_config(&self) -> Result<Value> {
        if !self.config_path.exists() {
            info!("配置文件不存在，创建默认配置");
            let default_config = utils::generate_default_machine_config();
            self.write_config(&default_config).await?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&self.config_path)
            .await
            .context("读取配置文件失败")?;

        let value: Value = serde_json::from_str(&content).context("解析配置文件失败")?;

        Ok(value)
    }

    /// 写入配置文件
    async fn write_config(&self, value: &Value) -> Result<()> {
        // 确保父目录存在
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("创建配置目录失败")?;
        }

        let content = serde_json::to_string_pretty(value).context("序列化配置失败")?;

        fs::write(&self.config_path, content)
            .await
            .context("写入配置文件失败")?;

        Ok(())
    }

    /// 获取机器 ID
    pub async fn get_machine_id(&self) -> Result<String> {
        let config = self.read_config().await?;
        Ok(config["telemetry.machineId"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    }

    /// 获取 Mac 机器 ID
    pub async fn get_mac_machine_id(&self) -> Result<String> {
        let config = self.read_config().await?;
        Ok(config["telemetry.macMachineId"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    }

    /// 获取设备 ID
    pub async fn get_dev_device_id(&self) -> Result<String> {
        let config = self.read_config().await?;
        Ok(config["telemetry.devDeviceId"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    }

    /// 获取 SQM ID
    pub async fn get_sqm_id(&self) -> Result<String> {
        let config = self.read_config().await?;
        Ok(config["telemetry.sqmId"]
            .as_str()
            .unwrap_or_default()
            .to_string())
    }

    /// 获取配置文件路径字符串
    pub async fn get_config_path_string(&self) -> Result<String> {
        Ok(self.config_path.to_string_lossy().to_string())
    }

    /// 备份配置
    pub async fn backup_config(&self) -> Result<()> {
        info!("开始备份配置文件");
        self.emit_progress("开始备份配置文件...");
        // 确保备份文件的父目录存在
        if let Some(parent) = self.backup_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context(format!("创建备份目录失败: {:?}", parent))?;
        }
        self.emit_progress("备份目录创建成功");
        // 删除旧的备份文件（如果存在）
        utils::remove_file_if_exists(&self.backup_path).await?;
        self.emit_progress("旧的备份文件删除成功");
        // 执行备份
        fs::copy(&self.config_path, &self.backup_path)
            .await
            .context(format!(
                "备份配置文件失败: 从 {:?} 复制到 {:?}",
                self.config_path, self.backup_path
            ))?;
        self.emit_progress("配置文件备份成功");
        // 设置备份文件为只读
        utils::set_file_permissions(&self.backup_path, true).await?;
        self.emit_progress("备份文件设置为只读成功");
        info!("配置文件备份完成");
        Ok(())
    }

    /// 还原配置
    pub async fn restore_config(&self) -> Result<()> {
        info!("开始还原配置文件");
        self.emit_progress("开始还原配置文件...");
        if !self.backup_path.exists() {
            warn!("备份文件不存在，无法还原");
            self.emit_progress("备份文件不存在，无法还原");
            return Err(anyhow::anyhow!("备份文件不存在"));
        }

        self.emit_progress("正在删除当前配置文件...");
        // 删除当前配置文件
        utils::remove_file_if_exists(&self.config_path).await?;
        self.emit_progress("当前配置文件删除成功");

        self.emit_progress("正在复制备份文件到配置文件...");
        // 复制备份文件到配置文件
        fs::copy(&self.backup_path, &self.config_path)
            .await
            .context(format!(
                "还原配置文件失败: 从 {:?} 复制到 {:?}",
                self.backup_path, self.config_path
            ))?;
        self.emit_progress("配置文件还原完成");
        info!("配置文件还原完成");
        Ok(())
    }

    /// 设置配置文件的只读属性
    pub async fn set_readonly(&self, readonly: bool) -> Result<()> {
        utils::set_file_permissions(&self.config_path, readonly).await
    }

    /// 更新机器 ID
    pub async fn update_machine_id(&self) -> Result<()> {
        // 1. 解除只读属性
        self.emit_progress("正在解除文件只读属性...");
        if let Err(e) = self.set_readonly(false).await {
            error!("解除只读属性失败: {}", e);
            return Err(e);
        }

        // 2. 备份当前配置
        self.emit_progress("正在备份当前配置...");
        if let Err(e) = self.backup_config().await {
            error!("备份配置失败: {}", e);
            // 备份失败后，恢复只读属性
            let _ = self.set_readonly(true).await;
            return Err(e);
        }

        // 3. 读取当前配置
        self.emit_progress("正在读取当前配置...");
        let mut config = match self.read_config().await {
            Ok(config) => config,
            Err(e) => {
                error!("读取配置失败: {}", e);
                // 读取失败后，恢复只读属性
                let _ = self.set_readonly(true).await;
                return Err(e);
            }
        };

        // 4. 生成新的配置
        self.emit_progress("正在生成新的机器ID配置...");
        let new_config = utils::generate_default_machine_config();

        // 5. 更新配置
        self.emit_progress("正在更新机器ID配置...");
        for (key, value) in new_config.as_object().unwrap() {
            config[key] = value.clone();
        }

        // 6. 写入配置
        self.emit_progress("正在写入新配置到文件...");
        if let Err(e) = self.write_config(&config).await {
            error!("写入配置失败: {}", e);
            // 写入失败后，恢复只读属性
            let _ = self.set_readonly(true).await;
            return Err(e);
        }

        // 等待文件系统完成写入
        sleep(Duration::from_millis(500)).await;

        // 7. 恢复只读属性
        self.emit_progress("正在恢复文件只读属性...");
        for _ in 0..3 {
            // 最多尝试3次
            if self.set_readonly(true).await.is_ok() {
                break;
            }
            sleep(Duration::from_millis(500)).await;
        }

        // 8. 验证文件权限
        self.emit_progress("正在验证文件权限...");
        let metadata = fs::metadata(&self.config_path).await?;
        let perms = metadata.permissions();
        if !perms.readonly() {
            error!("文件权限验证失败：文件不是只读状态");
            return Err(anyhow::anyhow!("文件权限验证失败：无法设置为只读状态"));
        }

        // 9. 更新 main.js 中的 ID
        self.emit_progress("正在更新 main.js 中的 ID...");

        // 先创建 patcher
        let mut patcher = crate::patcher::Patcher::new(None).map_err(|e| {
            error!("创建 Patcher 失败: {}", e);
            anyhow::anyhow!("创建 Patcher 失败: {}", e)
        })?;

        // 执行 patch 操作并等待结果
        patcher
            .patch(None, None, None, None)
            .await
            .with_context(|| {
                error!("应用补丁失败");
                "应用补丁失败"
            })?;

        self.emit_progress("机器ID更新完成！");
        info!("更新机器 ID 完成");
        Ok(())
    }

    /// 从注册表获取Cursor安装路径
    #[cfg(windows)]
    fn get_cursor_path() -> Option<String> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let subkey = r"SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\Cursor.exe";

        // 尝试打开注册表项
        if let Ok(cursor_key) = hklm.open_subkey(subkey) {
            // 获取默认值（安装路径）
            if let Ok(path) = cursor_key.get_value::<String, _>("") {
                return Some(path);
            }
        }

        // 如果上面失败，尝试从卸载信息中获取
        let uninstall_key = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
        if let Ok(uninstall) = hklm.open_subkey(uninstall_key) {
            // 使用 flatten() 简化遍历逻辑
            for key_name in uninstall.enum_keys().flatten() {
                // 使用 and_then 链式处理结果
                let cursor_path = uninstall.open_subkey(&key_name).ok().and_then(|app_key| {
                    app_key
                        .get_value::<String, _>("DisplayName")
                        .ok()
                        .filter(|name| name.contains("Cursor"))
                        .and_then(|_| app_key.get_value::<String, _>("InstallLocation").ok())
                        .map(|install_location| {
                            format!("{}\\Cursor.exe", install_location.trim_end_matches('\\'))
                        })
                });

                if let Some(path) = cursor_path {
                    return Some(path);
                }
            }
        }

        None
    }

    /// 从运行中的进程获取Cursor路径
    #[cfg(windows)]
    fn get_cursor_process_path() -> Option<String> {
        let mut system = System::new_all();
        system.refresh_all();

        // 查找所有cursor.exe进程
        for (pid, process) in system.processes() {
            if process.name().to_lowercase() == "cursor.exe" {
                if let Some(path) = process.exe().to_str() {
                    info!("Found running Cursor process (PID: {}): {}", pid, path);
                    return Some(path.to_string());
                }
            }
        }
        None
    }

    /// 重启 Cursor
    pub async fn restart_cursor(&self) -> Result<()> {
        self.emit_progress("开始重启 Cursor...");

        // Windows
        #[cfg(windows)]
        {
            // 先从运行中的进程获取路径
            self.emit_progress("正在查找运行中的Cursor进程...");
            let cursor_path = if let Some(path) = Self::get_cursor_process_path() {
                self.emit_progress(&format!("找到运行中的Cursor进程: {}", path));
                Some(path)
            } else {
                self.emit_progress("未找到运行中的Cursor进程，尝试从注册表查找...");
                // 尝试从注册表获取
                if let Some(path) = Self::get_cursor_path() {
                    info!("Found Cursor path in registry: {}", path);
                    self.emit_progress(&format!("在注册表中找到Cursor路径: {}", path));
                    if std::path::Path::new(&path).exists() {
                        Some(path)
                    } else {
                        self.emit_progress(&format!("注册表中的路径不存在: {}", path));
                        None
                    }
                } else {
                    self.emit_progress("注册表中未找到Cursor路径，尝试常见安装位置...");
                    let possible_paths = [
                        format!(
                            "{}\\Cursor\\Cursor.exe",
                            std::env::var("ProgramFiles")
                                .unwrap_or_else(|_| "C:\\Program Files".to_string())
                        ),
                        format!(
                            "{}\\Cursor\\Cursor.exe",
                            std::env::var("ProgramFiles(x86)")
                                .unwrap_or_else(|_| "C:\\Program Files (x86)".to_string())
                        ),
                        format!(
                            "{}\\Cursor\\Cursor.exe",
                            std::env::var("LocalAppData").unwrap_or_else(|_| format!(
                                "{}\\AppData\\Local",
                                std::env::var("USERPROFILE")
                                    .unwrap_or_else(|_| "C:\\Users\\Default".to_string())
                            ))
                        ),
                    ];

                    let mut found_path = None;
                    for path in possible_paths {
                        self.emit_progress(&format!("检查路径: {}", path));
                        if std::path::Path::new(&path).exists() {
                            self.emit_progress(&format!("找到Cursor: {}", path));
                            found_path = Some(path);
                            break;
                        }
                    }
                    found_path
                }
            };

            // 如果找不到Cursor路径，直接返回错误
            let cursor_path = match cursor_path {
                Some(path) => path,
                None => {
                    let err_msg = "未找到Cursor安装路径，请确认Cursor是否正确安装";
                    error!("{}", err_msg);
                    self.emit_progress(err_msg);
                    return Err(anyhow::anyhow!(err_msg));
                }
            };

            // 找到路径后，执行关闭操作
            self.emit_progress("正在尝试关闭Cursor进程...");
            match Command::new("wmic")
                .args(["process", "where", "name='cursor.exe'", "call", "terminate"])
                .output()
            {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    info!("WMIC terminate output: {}", stdout);
                    if !stderr.is_empty() {
                        error!("WMIC terminate error: {}", stderr);
                    }
                    self.emit_progress(&format!("WMIC terminate output: {}", stdout));
                }
                Err(e) => {
                    error!("Failed to terminate Cursor: {}", e);
                    self.emit_progress(&format!("关闭Cursor失败: {}", e));
                }
            }

            // 增加等待时间，确保进程完全退出
            self.emit_progress("等待进程完全退出...");
            sleep(Duration::from_secs(5)).await;

            // 验证进程是否完全退出
            let sys = System::new_all();
            let mut retry_count = 0;
            while retry_count < 3 {
                let cursor_running = sys
                    .processes()
                    .values()
                    .any(|process| process.name().to_lowercase() == "cursor.exe");

                if !cursor_running {
                    break;
                }

                retry_count += 1;
                self.emit_progress("进程仍在运行，等待额外时间...");
                sleep(Duration::from_secs(2)).await;
            }

            // 启动 Cursor
            self.emit_progress("正在启动 Cursor...");
            match Command::new(&cursor_path).spawn() {
                Ok(_) => {
                    self.emit_progress("Cursor 启动成功");
                }
                Err(e) => {
                    error!("启动 Cursor 失败: {}", e);
                    self.emit_progress(&format!("启动 Cursor 失败: {}", e));
                    return Err(anyhow::anyhow!("启动 Cursor 失败"));
                }
            }
        }

        // macOS
        #[cfg(target_os = "macos")]
        {
            // 关闭 Cursor
            Command::new("pkill")
                .args(["-x", "Cursor"])
                .output()
                .context("关闭 Cursor 失败")?;

            sleep(Duration::from_secs(5)).await;

            // 启动 Cursor
            Command::new("open")
                .args(["-a", "Cursor"])
                .spawn()
                .context("启动 Cursor 失败")?;
        }

        // Linux
        #[cfg(target_os = "linux")]
        {
            // 优雅关闭 Cursor
            Command::new("killall")
                .args(["-TERM", "cursor"])
                .output()
                .context("关闭 Cursor 失败")?;

            sleep(Duration::from_secs(5)).await;

            // 验证进程是否完全退出
            let mut retry_count = 0;
            while retry_count < 3 {
                match Command::new("pgrep").arg("cursor").output() {
                    Ok(output) => {
                        if output.status.success() {
                            retry_count += 1;
                            self.emit_progress("进程仍在运行，等待额外时间...");
                            sleep(Duration::from_secs(2)).await;
                        } else {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }

            // 尝试多个可能的启动路径
            let possible_paths = [
                "/usr/bin/cursor",
                "/usr/local/bin/cursor",
                "/opt/cursor/cursor",
            ];

            let mut started = false;
            for path in possible_paths {
                if std::path::Path::new(path).exists() {
                    Command::new(path).spawn().context("启动 Cursor 失败")?;
                    started = true;
                    break;
                }
            }

            if !started {
                return Err(anyhow::anyhow!("找不到 Cursor 可执行文件"));
            }
        }

        self.emit_progress("Cursor 重启操作完成！");
        Ok(())
    }

    /// 重置机器 ID 并重启Cursor
    pub async fn reset_machine_id(&self) -> Result<()> {
        self.emit_progress("开始重置机器 ID");

        // 重置认证信息
        self.emit_progress("开始重置认证信息...");
        if !auth_manager::reset_auth(&*self.progress_emitter).await {
            return Err(anyhow::anyhow!("重置认证信息失败"));
        }
        self.emit_progress("认证信息重置成功");

        // 更新机器ID
        self.emit_progress("开始更新机器ID...");
        self.update_machine_id().await?;

        // 重启Cursor
        self.emit_progress("开始重启Cursor...");
        self.restart_cursor().await?;

        self.emit_progress("重置机器ID完成！");
        Ok(())
    }

    /// 获取所有 ID 信息
    pub async fn get_all_ids(&self) -> Result<(String, String, String, String, String)> {
        let machine_id = self.get_machine_id().await?;
        let mac_machine_id = self.get_mac_machine_id().await?;
        let dev_device_id = self.get_dev_device_id().await?;
        let sqm_id = self.get_sqm_id().await?;
        let config_path = self.get_config_path_string().await?;

        Ok((
            machine_id,
            mac_machine_id,
            dev_device_id,
            sqm_id,
            config_path,
        ))
    }
}
