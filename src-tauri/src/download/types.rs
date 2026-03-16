use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use serde::Serialize;

// ── Shared cancel flag ────────────────────────────────────────────────────────

pub struct DownloadState {
    pub cancel: Arc<AtomicBool>,
}

impl Default for DownloadState {
    fn default() -> Self {
        Self {
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl DownloadState {
    pub fn reset(&self) {
        self.cancel.store(false, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
}

// ── Event payloads ────────────────────────────────────────────────────────────

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub model_id: String,
    pub filename: String,
    pub downloaded: u64,
    pub total: u64,
    pub percent: f64,
}