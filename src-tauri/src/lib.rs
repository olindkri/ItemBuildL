mod game_state;
mod engine;
mod poller;

#[tauri::command]
fn toggle_always_on_top(window: tauri::WebviewWindow, enable: bool) -> Result<(), String> {
    window.set_always_on_top(enable).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![toggle_always_on_top])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                poller::start_polling(app_handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
