use serde_json::json;
use tauri::{AppHandle, Emitter};
use tracing::info;

/// 进度事件发送器 trait
pub trait ProgressEmitter: Send + Sync {
    fn emit_progress(&self, message: &str);
    fn clone_box(&self) -> Box<dyn ProgressEmitter>;
}

impl Clone for Box<dyn ProgressEmitter> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// 空进度事件发送器（不执行任何操作）
#[derive(Clone)]
pub struct NoopProgressEmitter;

impl NoopProgressEmitter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoopProgressEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressEmitter for NoopProgressEmitter {
    fn emit_progress(&self, _message: &str) {
        // 不执行任何操作
    }

    fn clone_box(&self) -> Box<dyn ProgressEmitter> {
        Box::new(self.clone())
    }
}

/// Tauri 进度事件发送器
#[derive(Clone)]
pub struct TauriProgressEmitter {
    app_handle: AppHandle,
}

impl TauriProgressEmitter {
    /// 创建新的 Tauri 进度事件发送器
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl ProgressEmitter for TauriProgressEmitter {
    fn emit_progress(&self, message: &str) {
        info!("发送进度事件: {}", message);
        if let Err(e) = self.app_handle.emit(
            "reset_progress",
            json!({
                "message": message
            }),
        ) {
            info!("发送进度事件失败: {}", e);
        }
    }

    fn clone_box(&self) -> Box<dyn ProgressEmitter> {
        Box::new(self.clone())
    }
}
