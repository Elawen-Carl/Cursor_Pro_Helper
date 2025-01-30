// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use cursor_pro_helper::commands::*;

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
