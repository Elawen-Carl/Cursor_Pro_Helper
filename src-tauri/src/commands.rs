use crate::api_config::{ApiConfig, ApiConfigManager};
use crate::auth_manager;
use crate::events::TauriProgressEmitter;
use crate::machine::MachineService;
use serde::{Deserialize, Serialize};
use tauri::command;
use tracing::{error, info};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MachineIds {
    machine_id: String,
    mac_machine_id: String,
    dev_device_id: String,
    sqm_id: String,
    config_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MainJsIds {
    machine_id: String,
    mac_machine_id: String,
    dev_device_id: String,
    sqm_id: String,
    js_path: String,
}

#[command]
pub async fn get_all_ids() -> Result<MachineIds, String> {
    info!("开始获取所有 ID");
    let machine_service = MachineService::new().await.map_err(|e| {
        error!("创建机器服务失败: {}", e);
        e.to_string()
    })?;

    let (machine_id, mac_machine_id, dev_device_id, sqm_id, config_path) =
        machine_service.get_all_ids().await.map_err(|e| {
            error!("获取所有 ID 失败: {}", e);
            e.to_string()
        })?;

    let result = MachineIds {
        machine_id,
        mac_machine_id,
        dev_device_id,
        sqm_id,
        config_path,
    };

    info!("返回给前端的数据: {:?}", result);
    Ok(result)
}

#[command]
pub async fn get_machine_id() -> Result<String, String> {
    info!("开始获取机器 ID");
    let machine_service = MachineService::new().await.map_err(|e| {
        error!("创建机器服务失败: {}", e);
        e.to_string()
    })?;
    machine_service.get_machine_id().await.map_err(|e| {
        error!("获取机器 ID 失败: {}", e);
        e.to_string()
    })
}

#[command]
pub async fn reset_machine_id(app_handle: tauri::AppHandle) -> Result<(), String> {
    info!("开始重置机器 ID");
    let progress_emitter = Box::new(TauriProgressEmitter::new(app_handle));
    let machine_service = MachineService::with_progress(progress_emitter)
        .await
        .map_err(|e| {
            error!("创建机器服务失败: {}", e);
            e.to_string()
        })?;

    machine_service.reset_machine_id().await.map_err(|e| {
        error!("重置机器 ID 失败: {}", e);
        e.to_string()
    })
}

#[command]
pub async fn backup_config(app_handle: tauri::AppHandle) -> Result<(), String> {
    info!("开始备份配置");
    let progress_emitter = Box::new(TauriProgressEmitter::new(app_handle));
    let machine_service = MachineService::with_progress(progress_emitter)
        .await
        .map_err(|e| {
            error!("创建机器服务失败: {}", e);
            e.to_string()
        })?;

    // 备份配置文件
    machine_service.backup_config().await.map_err(|e| {
        error!("备份配置文件失败: {}", e);
        e.to_string()
    })?;

    // 备份 main.js
    let patcher = crate::patcher::Patcher::new(None).map_err(|e| {
        error!("创建 Patcher 失败: {}", e);
        e.to_string()
    })?;

    patcher.backup().await.map_err(|e| {
        error!("备份 main.js 失败: {}", e);
        e.to_string()
    })?;

    Ok(())
}

#[command]
pub async fn restore_config(app_handle: tauri::AppHandle) -> Result<(), String> {
    info!("开始还原配置");
    let progress_emitter = Box::new(TauriProgressEmitter::new(app_handle));
    let machine_service = MachineService::with_progress(progress_emitter.clone())
        .await
        .map_err(|e| {
            error!("创建机器服务失败: {}", e);
            e.to_string()
        })?;

    // 还原配置文件
    machine_service.restore_config().await.map_err(|e| {
        error!("还原配置文件失败: {}", e);
        e.to_string()
    })?;

    // 还原 main.js
    let patcher = crate::patcher::Patcher::new(None).map_err(|e| {
        error!("创建 Patcher 失败: {}", e);
        e.to_string()
    })?;

    patcher.restore().await.map_err(|e| {
        error!("还原 main.js 失败: {}", e);
        e.to_string()
    })?;

    Ok(())
}

#[command]
pub async fn update_machine_id(app_handle: tauri::AppHandle) -> Result<(), String> {
    info!("开始更新机器 ID");
    let progress_emitter = Box::new(TauriProgressEmitter::new(app_handle));
    let machine_service = MachineService::with_progress(progress_emitter)
        .await
        .map_err(|e| {
            error!("创建机器服务失败: {}", e);
            e.to_string()
        })?;

    // 更新配置文件中的 ID
    machine_service.update_machine_id().await.map_err(|e| {
        error!("更新机器 ID 失败: {}", e);
        e.to_string()
    })?;
    Ok(())
}

#[command]
pub async fn update_auth(email: String, token: String) -> Result<(), String> {
    info!("开始更新认证信息");
    match auth_manager::update_auth(Some(email), Some(token.clone()), Some(token)) {
        true => Ok(()),
        false => Err("更新认证信息失败".to_string()),
    }
}

#[command]
pub async fn reset_auth(app_handle: tauri::AppHandle) -> bool {
    let progress_emitter = Box::new(TauriProgressEmitter::new(app_handle));
    auth_manager::reset_auth(&*progress_emitter).await
}

#[tauri::command]
pub fn get_api_config() -> Result<ApiConfig, String> {
    let manager = ApiConfigManager::new().map_err(|e| e.to_string())?;
    manager.load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_api_config(url: String) -> Result<(), String> {
    let manager = ApiConfigManager::new().map_err(|e| e.to_string())?;
    let config = ApiConfig { url };
    manager.save(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_api_config() -> Result<ApiConfig, String> {
    let manager = ApiConfigManager::new().map_err(|e| e.to_string())?;
    manager.reset().map_err(|e| e.to_string())
}

#[command]
pub fn get_mainjs_ids() -> Result<MainJsIds, String> {
    info!("开始获取 main.js 中的 ID");
    let patcher = crate::patcher::Patcher::new(None).map_err(|e| {
        error!("创建 Patcher 失败: {}", e);
        e.to_string()
    })?;

    let (machine_id, mac_machine_id, dev_device_id, sqm_id) =
        patcher.get_current_ids().map_err(|e| {
            error!("获取 main.js 中的 ID 失败: {}", e);
            e.to_string()
        })?;

    Ok(MainJsIds {
        machine_id,
        mac_machine_id,
        dev_device_id,
        sqm_id,
        js_path: patcher.get_js_path().to_string_lossy().to_string(),
    })
}
