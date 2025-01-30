//! 工具函数模块

use rand::{thread_rng, RngCore};
use std::fmt::Write;
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
