//! Persistent storage for the HuggingFace token and downloaded model registry.
//!
//! All reads and writes go through [`tauri_plugin_store`], which encrypts the
//! JSON file on disk. The token is never returned to the frontend after being
//! saved — callers can only query its presence via [`has_token`].

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "codestrux.json";
const TOKEN_KEY: &str = "hf_token";
const MODELS_KEY: &str = "downloaded_models";

// ── Token management ──────────────────────────────────────────────────────────

/// Persists the HuggingFace token to the encrypted on-disk store.
///
/// # Errors
///
/// Returns an error if the store cannot be opened or flushed to disk.
#[tauri::command]
pub fn save_token(app: AppHandle, token: String) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set(TOKEN_KEY, serde_json::Value::String(token));
    store.save().map_err(|e| e.to_string())
}

/// Returns `true` if a non-empty token is stored.
///
/// The frontend uses this to decide whether to show the "add token" prompt;
/// it never receives the token value itself.
#[tauri::command]
pub fn has_token(app: AppHandle) -> bool {
    app.store(STORE_FILE)
        .ok()
        .and_then(|s| s.get(TOKEN_KEY))
        .and_then(|v| v.as_str().map(|s| !s.is_empty()))
        .unwrap_or(false)
}

/// Deletes the stored token.
///
/// # Errors
///
/// Returns an error if the store cannot be opened or flushed to disk.
#[tauri::command]
pub fn delete_token(app: AppHandle) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.delete(TOKEN_KEY);
    store.save().map_err(|e| e.to_string())
}

/// Returns the stored token for internal use.
///
/// Not exposed as a Tauri command — the token must never be sent back to the
/// frontend.
pub fn read_token(app: &AppHandle) -> Option<String> {
    app.store(STORE_FILE)
        .ok()
        .and_then(|s| s.get(TOKEN_KEY))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .filter(|s| !s.is_empty())
}

// ── Downloaded model registry ─────────────────────────────────────────────────

/// A model file that has been fully downloaded and registered.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StoredModel {
    pub model_id: String,
    pub filename: String,
    /// Absolute path to the `.gguf` file on disk.
    pub path: String,
    /// File size in bytes.
    pub size: u64,
}

/// Persists a downloaded model entry, upserting by `model_id` + `filename`.
pub fn save_downloaded_model(app: &AppHandle, model: StoredModel) {
    let Ok(store) = app.store(STORE_FILE) else {
        return;
    };
    let mut models = get_downloaded_models_internal(app);
    if let Some(pos) = models
        .iter()
        .position(|m| m.model_id == model.model_id && m.filename == model.filename)
    {
        models[pos] = model;
    } else {
        models.push(model);
    }
    store.set(
        MODELS_KEY,
        serde_json::to_value(&models).unwrap_or_default(),
    );
    let _ = store.save();
}

/// Reads the downloaded models list for internal use.
///
/// Returns an empty list if the store cannot be opened or the key is absent.
pub fn get_downloaded_models_internal(app: &AppHandle) -> Vec<StoredModel> {
    app.store(STORE_FILE)
        .ok()
        .and_then(|s| s.get(MODELS_KEY))
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Returns all downloaded models to the frontend.
#[tauri::command]
pub fn get_downloaded_models(app: AppHandle) -> Vec<StoredModel> {
    get_downloaded_models_internal(&app)
}

/// Removes a model from the registry and deletes its file from disk.
///
/// Silently succeeds if the model is not found in the registry. File removal
/// is best-effort — a failure there does not fail the command.
///
/// # Errors
///
/// Returns an error if the store cannot be opened or flushed to disk.
#[tauri::command]
pub fn delete_downloaded_model(
    app: AppHandle,
    model_id: String,
    filename: String,
) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    let mut models = get_downloaded_models_internal(&app);

    if let Some(pos) = models
        .iter()
        .position(|m| m.model_id == model_id && m.filename == filename)
    {
        let path = models[pos].path.clone();
        models.remove(pos);
        store.set(
            MODELS_KEY,
            serde_json::to_value(&models).unwrap_or_default(),
        );
        store.save().map_err(|e| e.to_string())?;
        let _ = std::fs::remove_file(&path);
    }

    Ok(())
}