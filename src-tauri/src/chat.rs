//! Streaming chat completion against the HuggingFace Inference API.
//!
//! Exposes two Tauri commands: [`start_chat`] opens a streaming SSE connection
//! and [`stop_chat`] signals it to close after the current chunk.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::store::read_token;

// ── Shared state ──────────────────────────────────────────────────────────────

/// Tauri-managed state for the chat subsystem.
///
/// A single cancel flag is sufficient because the UI blocks sending while a
/// response is streaming, ensuring only one request runs at a time.
///
/// The [`reqwest::Client`] is held here so that connection pooling and TLS
/// sessions to `api-inference.huggingface.co` are reused across requests.
pub struct ChatState {
    pub cancel: Arc<AtomicBool>,
    pub client: reqwest::Client,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            cancel: Arc::new(AtomicBool::new(false)),
            // `Accept-Encoding: identity` prevents transparent decompression of
            // the SSE stream, which would corrupt chunked delivery.
            client: reqwest::Client::builder()
                .default_headers({
                    let mut h = reqwest::header::HeaderMap::new();
                    h.insert(
                        reqwest::header::ACCEPT_ENCODING,
                        "identity".parse().unwrap(),
                    );
                    h
                })
                .build()
                .expect("Failed to build chat HTTP client"),
        }
    }
}

// ── Types ─────────────────────────────────────────────────────────────────────

/// A single message in a chat conversation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    /// `"user"` or `"assistant"`.
    pub role: String,
    pub content: String,
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Begin a streaming chat completion against the HuggingFace Inference API.
///
/// Emits the following events to the frontend:
///
/// | Event        | Payload                  |
/// |--------------|--------------------------|
/// | `chat-token` | `String` — partial token |
/// | `chat-done`  | `()`                     |
/// | `chat-error` | `String` — error message |
///
/// # Errors
///
/// Returns an error if no token is stored, the HTTP request fails, or the
/// server returns a non-success status.
#[tauri::command]
pub async fn start_chat(
    app: AppHandle,
    state: State<'_, ChatState>,
    model: String,
    messages: Vec<Message>,
) -> Result<(), String> {
    state.cancel.store(false, Ordering::SeqCst);
    let cancel = Arc::clone(&state.cancel);

    // Token is read server-side so it is never exposed to the frontend.
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

    // Serialized manually with `.body()` because the reqwest `json` feature is
    // not enabled — the download module does not need it and enabling features
    // for a single call site is unnecessary.
    let body = serde_json::to_string(&serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "max_tokens": 2048,
    }))
    .map_err(|e| e.to_string())?;

    let response = state
        .client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .body(body)
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

/// Signals the running stream to stop after the current chunk.
#[tauri::command]
pub fn stop_chat(state: State<'_, ChatState>) {
    state.cancel.store(true, Ordering::SeqCst);
}