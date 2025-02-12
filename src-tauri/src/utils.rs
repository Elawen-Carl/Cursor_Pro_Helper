//! 工具函数模块

use anyhow::{Context, Result};
use rand::{thread_rng, RngCore};
use std::fmt::Write;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

/// 生成64位十六进制字符串
pub fn generate_hex_string() -> String {
    let mut rng = thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    bytes.iter().fold(String::with_capacity(64), |mut acc, b| {
        let _ = write!(acc, "{:02x}", b);
        acc
    })
}

/// 生成标准的机器ID（改为64位十六进制）
pub fn generate_machine_id() -> String {
    let mut rng = thread_rng();
    let mut bytes = [0u8; 32];
    rng.fill_bytes(&mut bytes);
    bytes.iter().fold(String::with_capacity(64), |mut acc, b| {
        let _ = write!(acc, "{:02x}", b);
        acc
    })
}

/// 生成UUID（用于devDeviceId）
pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

/// 生成标准的 SQM ID（改为大括号包裹的大写UUID）
pub fn generate_sqm_id() -> String {
    format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase())
}

/// 生成默认的机器配置
pub fn generate_default_machine_config() -> serde_json::Value {
    let machine_id = generate_machine_id();
    let mac_machine_id = generate_machine_id();
    let dev_device_id = generate_uuid();
    let sqm_id = generate_sqm_id();
    let telemetry_machine_id = generate_machine_id(); // 复用同一个生成器

    serde_json::json!({
        "machineId": machine_id,
        "telemetry.machineId": telemetry_machine_id,
        "telemetry.macMachineId": mac_machine_id,
        "telemetry.devDeviceId": dev_device_id,
        "telemetry.sqmId": sqm_id
    })
}

/// 设置文件的权限
pub async fn set_file_permissions(file_path: &PathBuf, readonly: bool) -> Result<()> {
    let mut perms = fs::metadata(file_path).await?.permissions();

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

    fs::set_permissions(file_path, perms)
        .await
        .context(format!("设置文件权限失败: {:?}", file_path))?;

    Ok(())
}

/// 删除文件，如果文件存在且为只读，先解除只读属性
pub async fn remove_file_if_exists(file_path: &PathBuf) -> Result<()> {
    if file_path.exists() {
        // 先解除只读属性
        set_file_permissions(file_path, false).await?;
        // 然后删除文件
        fs::remove_file(file_path)
            .await
            .context(format!("删除文件失败: {:?}", file_path))?;
    }
    Ok(())
}
