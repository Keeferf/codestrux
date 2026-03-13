use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_FILE: &str = "codestrux.json";
const TOKEN_KEY: &str = "hf_token";

/// Persist the HF token to the encrypted on-disk store.
/// The token never passes back to the frontend after being saved.
#[tauri::command]
pub fn save_token(app: AppHandle, token: String) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set(TOKEN_KEY, serde_json::Value::String(token));
    store.save().map_err(|e| e.to_string())
}

/// Returns true if a non-empty token is stored — frontend only needs to
/// know whether to show the "add token" prompt, never the value itself.
#[tauri::command]
pub fn has_token(app: AppHandle) -> bool {
    app.store(STORE_FILE)
        .ok()
        .and_then(|s| s.get(TOKEN_KEY))
        .and_then(|v| v.as_str().map(|s| !s.is_empty()))
        .unwrap_or(false)
}

/// Wipe the stored token.
#[tauri::command]
pub fn delete_token(app: AppHandle) -> Result<(), String> {
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.delete(TOKEN_KEY);
    store.save().map_err(|e| e.to_string())
}

/// Read the token internally (not exposed as a Tauri command).
pub fn read_token(app: &AppHandle) -> Option<String> {
    app.store(STORE_FILE)
        .ok()
        .and_then(|s| s.get(TOKEN_KEY))
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .filter(|s| !s.is_empty())
}