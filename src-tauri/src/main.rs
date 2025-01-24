// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use cursor_pro_helper::commands::*;
use cursor_pro_helper::auth_manager;
use cursor_pro_helper::events::TauriProgressEmitter;

#[tauri::command]
fn update_auth(
    email: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
) -> bool {
    auth_manager::update_auth(email, access_token, refresh_token)
}

#[tauri::command]
async fn reset_auth(app_handle: tauri::AppHandle) -> bool {
    let progress_emitter = Box::new(TauriProgressEmitter::new(app_handle));
    auth_manager::reset_auth(&*progress_emitter).await
}

fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_all_ids,
            get_machine_id,
            reset_machine_id,
            backup_config,
            restore_config,
            update_machine_id,
            update_auth,
            reset_auth,
            get_api_config,
            save_api_config,
            reset_api_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
