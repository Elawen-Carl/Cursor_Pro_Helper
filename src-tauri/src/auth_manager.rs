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
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ApiResponse {
    success: bool,
    data: AccountData,
    message: String,
}

pub async fn reset_auth() -> bool {
    info!("Starting auth reset process");

    // 获取随机账号
    let client = reqwest::Client::new();
    let response = match client
        .get("https://cursor-account-api.vercel.app/account/random")
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to fetch random account: {}", e);
            return false;
        }
    };

    // 打印响应内容以便调试
    let response_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            error!("Failed to get response text: {}", e);
            return false;
        }
    };
    info!("API Response: {}", response_text);

    // 解析JSON
    let api_response: ApiResponse = match serde_json::from_str(&response_text) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to parse API response: {}", e);
            return false;
        }
    };

    if !api_response.success {
        error!("API returned failure: {}", api_response.message);
        return false;
    }

    // 更新认证信息
    update_auth(
        Some(api_response.data.email),
        Some(api_response.data.token.clone()),
        Some(api_response.data.token),
    )
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
