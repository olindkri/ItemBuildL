mod game_state;
mod engine;
mod poller;
mod ai_advisor;

#[tauri::command]
fn toggle_always_on_top(window: tauri::WebviewWindow, enable: bool) -> Result<(), String> {
    window.set_always_on_top(enable).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();
    let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![toggle_always_on_top])
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                poller::start_polling(app_handle, api_key).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
