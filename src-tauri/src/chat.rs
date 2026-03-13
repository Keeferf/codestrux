use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::store::read_token;

// ── Shared cancel flag ────────────────────────────────────────────────────────

/// Held in Tauri's managed state. A single flag is enough because only one
/// streaming request runs at a time (the UI blocks sending while loading).
pub struct ChatState {
    pub cancel: Arc<AtomicBool>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Begin a streaming chat completion.
///
/// Emits these events to the frontend:
///   - `chat-token`  { content: String }
///   - `chat-done`   {}
///   - `chat-error`  { message: String }
#[tauri::command]
pub async fn start_chat(
    app: AppHandle,
    state: State<'_, ChatState>,
    model: String,
    messages: Vec<Message>,
) -> Result<(), String> {
    // Reset the cancel flag before every new request
    state.cancel.store(false, Ordering::SeqCst);
    let cancel = Arc::clone(&state.cancel);

    // Pull token from the secure store — never from JS
    let token = match read_token(&app) {
        Some(t) => t,
        None => {
            let _ = app.emit("chat-error", "No HuggingFace token saved. Add one in Settings.");
            return Err("no token".into());
        }
    };

    let url = format!(
        "https://api-inference.huggingface.co/models/{}/v1/chat/completions",
        model
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
            "max_tokens": 2048,
        }))
        .send()
        .await
        .map_err(|e| {
            let msg = format!("Request failed: {}", e);
            let _ = app.emit("chat-error", &msg);
            msg
        })?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        let msg = format!("HF API error {}: {}", status, body);
        let _ = app.emit("chat-error", &msg);
        return Err(msg);
    }

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        // Honour a stop request from the frontend
        if cancel.load(Ordering::SeqCst) {
            let _ = app.emit("chat-done", ());
            return Ok(());
        }

        let chunk = chunk.map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&chunk);

        for line in text.lines() {
            if !line.starts_with("data: ") {
                continue;
            }
            let data = line[6..].trim();
            if data == "[DONE]" {
                let _ = app.emit("chat-done", ());
                return Ok(());
            }
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                    if !content.is_empty() {
                        let _ = app.emit("chat-token", content);
                    }
                }
            }
        }
    }

    let _ = app.emit("chat-done", ());
    Ok(())
}

/// Signal the running stream to stop after the current chunk.
#[tauri::command]
pub fn stop_chat(state: State<'_, ChatState>) {
    state.cancel.store(true, Ordering::SeqCst);
}