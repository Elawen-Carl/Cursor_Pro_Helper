use crate::events::ProgressEmitter;
use rusqlite::{Connection, Result as SqliteResult};
use serde::Deserialize;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AccountData {
    email: String,
    password: String,
    token: String,
    usage_limit: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ApiResponse {
    success: bool,
    data: AccountData,
    message: String,
}

/// 删除账号
pub async fn delete_account(email: &str, progress_emitter: &dyn ProgressEmitter) -> bool {
    info!("Starting account deletion process for email: {}", email);
    progress_emitter.emit_progress(&format!("开始删除账号: {}", email));

    let client = reqwest::Client::new();
    let url = format!("https://cursor-account-api.vercel.app/account/{}", email);

    match client.delete(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                info!("Successfully deleted account for email: {}", email);
                progress_emitter.emit_progress("账号删除成功");
                true
            } else {
                let error_msg = format!("删除账号失败. 状态码: {}", response.status());
                error!("{}", error_msg);
                progress_emitter.emit_progress(&error_msg);
                false
            }
        }
        Err(e) => {
            let error_msg = format!("发送删除请求失败: {}", e);
            error!("{}", error_msg);
            progress_emitter.emit_progress(&error_msg);
            false
        }
    }
}

pub async fn reset_auth(progress_emitter: &dyn ProgressEmitter) -> bool {
    info!("Starting auth reset process");
    progress_emitter.emit_progress("开始重置认证信息...");

    // 获取随机账号
    progress_emitter.emit_progress("正在获取随机账号...");
    let client = reqwest::Client::new();
    let response = match client
        .get("https://cursor-account-api.vercel.app/account/random")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            let error_msg = format!("获取随机账号失败: {}", e);
            error!("{}", error_msg);
            progress_emitter.emit_progress(&error_msg);
            return false;
        }
    };

    // 打印响应内容以便调试
    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            let error_msg = format!("获取响应内容失败: {}", e);
            error!("{}", error_msg);
            progress_emitter.emit_progress(&error_msg);
            return false;
        }
    };
    info!("API Response: {}", response_text);

    // 解析JSON
    let api_response: ApiResponse = match serde_json::from_str(&response_text) {
        Ok(data) => data,
        Err(e) => {
            let error_msg = format!("解析API响应失败: {}", e);
            error!("{}", error_msg);
            progress_emitter.emit_progress(&error_msg);
            return false;
        }
    };

    if !api_response.success {
        let error_msg = format!("API返回失败: {}", api_response.message);
        error!("{}", error_msg);
        progress_emitter.emit_progress(&error_msg);
        return false;
    }

    progress_emitter.emit_progress("成功获取随机账号");

    // 更新认证信息
    progress_emitter.emit_progress("开始更新认证信息...");
    let email = api_response.data.email.clone();
    let success = update_auth(
        Some(email.clone()),
        Some(api_response.data.token.clone()),
        Some(api_response.data.token),
    );

    if !success {
        progress_emitter.emit_progress("更新认证信息失败");
        return false;
    }

    progress_emitter.emit_progress("认证信息更新成功");

    // 如果更新成功，删除旧账号
    if success && !delete_account(&email, progress_emitter).await {
        error!("Failed to delete account after reset");
        progress_emitter.emit_progress("删除旧账号失败，但认证信息已更新");
        // 即使删除失败，我们仍然继续，因为认证信息已经更新
    }

    progress_emitter.emit_progress("重置认证信息完成");
    success
}

pub fn update_auth(
    email: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
) -> bool {
    let db_path = get_db_path();

    // Ensure directory exists
    if let Some(dir) = db_path.parent() {
        if let Err(e) = fs::create_dir_all(dir) {
            error!("Failed to create directory: {}", e);
            return false;
        }
        // Set directory permissions on Unix systems
        #[cfg(unix)]
        if let Err(e) = fs::set_permissions(dir, fs::Permissions::from_mode(0o755)) {
            error!("Failed to set directory permissions: {}", e);
            // Continue anyway as this is not critical
        }
    }

    // Initialize database if it doesn't exist
    if !db_path.exists() {
        if let Err(e) = initialize_db(&db_path) {
            error!("Failed to initialize database: {}", e);
            return false;
        }
        // Set file permissions on Unix systems
        #[cfg(unix)]
        if let Err(e) = fs::set_permissions(&db_path, fs::Permissions::from_mode(0o644)) {
            error!("Failed to set file permissions: {}", e);
            // Continue anyway as this is not critical
        }
    }

    match update_auth_internal(db_path, email, access_token, refresh_token) {
        Ok(_) => true,
        Err(e) => {
            match e {
                rusqlite::Error::SqliteFailure(_, Some(msg)) => {
                    error!("SQLite error: {}", msg);
                }
                rusqlite::Error::SqliteFailure(err, None) => {
                    error!("SQLite error code: {}", err);
                }
                _ => error!("Database error: {}", e),
            }
            false
        }
    }
}

fn get_db_path() -> PathBuf {
    if cfg!(windows) {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            PathBuf::from(appdata)
                .join("Cursor")
                .join("User")
                .join("globalStorage")
                .join("state.vscdb")
        } else {
            panic!("APPDATA environment variable not set");
        }
    } else if cfg!(target_os = "macos") {
        dirs::home_dir()
            .expect("Could not find home directory")
            .join("Library")
            .join("Application Support")
            .join("Cursor")
            .join("User")
            .join("globalStorage")
            .join("state.vscdb")
    } else {
        // Linux
        dirs::home_dir()
            .expect("Could not find home directory")
            .join(".config")
            .join("Cursor")
            .join("User")
            .join("globalStorage")
            .join("state.vscdb")
    }
}

fn initialize_db(db_path: &PathBuf) -> SqliteResult<()> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ItemTable (
            key TEXT PRIMARY KEY,
            value TEXT
        )",
        [],
    )?;
    Ok(())
}

fn update_auth_internal(
    db_path: PathBuf,
    email: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
) -> SqliteResult<()> {
    let mut conn = Connection::open(db_path)?;

    // Set pragmas for better performance
    conn.execute_batch(
        "
        PRAGMA busy_timeout = 5000;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
    ",
    )?;

    let tx = conn.transaction()?;

    let mut updates = Vec::new();
    if let Some(email) = email {
        updates.push(("cursorAuth/cachedEmail", email));
    }
    if let Some(token) = access_token {
        updates.push(("cursorAuth/accessToken", token));
    }
    if let Some(token) = refresh_token {
        updates.push(("cursorAuth/refreshToken", token));
        updates.push(("cursorAuth/cachedSignUpType", "Auth_0".to_string()));
    }

    let result: SqliteResult<()> = (|| {
        for (key, value) in updates {
            let count: i64 = tx.query_row(
                "SELECT COUNT(*) FROM ItemTable WHERE key = ?",
                [key],
                |row| row.get(0),
            )?;

            if count == 0 {
                tx.execute(
                    "INSERT INTO ItemTable (key, value) VALUES (?, ?)",
                    [key, &value],
                )?;
            } else {
                tx.execute(
                    "UPDATE ItemTable SET value = ? WHERE key = ?",
                    [&value, key],
                )?;
            }
            info!("Updating {}", key.split('/').last().unwrap_or(key));
        }
        tx.commit()?;
        info!("Database updated successfully");
        Ok(())
    })();

    // Ensure connection is closed even if there's an error
    drop(conn);

    result
}
