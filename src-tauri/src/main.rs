// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use cursor_pro_helper::commands::*;
mod auth_manager;

#[tauri::command]
fn update_auth(
    email: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
) -> bool {
    auth_manager::update_auth(email, access_token, refresh_token)
}

#[tauri::command]
async fn reset_auth() -> bool {
    auth_manager::reset_auth().await
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
