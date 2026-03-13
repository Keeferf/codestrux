mod chat;
mod hardware;
mod store;

use chat::ChatState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(ChatState::default())
        .invoke_handler(tauri::generate_handler![
            // hardware
            hardware::get_hardware_info,
            // token management
            store::save_token,
            store::has_token,
            store::delete_token,
            // chat
            chat::start_chat,
            chat::stop_chat,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}