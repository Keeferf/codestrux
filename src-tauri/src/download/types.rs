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
    /// Resets the cancel flag so a new download can begin.
    pub fn reset(&self) {
        self.cancel.store(false, Ordering::SeqCst);
    }

    /// Signals any in-progress download to stop.
    /// Mirrors reset() and is_cancelled() so callers never need to
    /// reach into the field directly.
    pub fn request_cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    /// Returns a cloned Arc of the cancel flag for passing into download tasks.
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel)
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