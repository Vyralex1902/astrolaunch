#[tauri::command]
pub fn minimize_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
    }
}

#[tauri::command]
pub fn maximize_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.maximize();
    }
}

#[tauri::command]
pub fn resize_window_80(app: tauri::AppHandle) {
    use tauri::Size;
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(monitor) = window.primary_monitor() {
            if let Some(m) = monitor {
                let size = m.size();
                let new_width = (size.width as f64 * 0.8) as f64;
                let new_height = (size.height as f64 * 0.8) as f64;
                let _ = window.set_size(Size::Logical(tauri::LogicalSize {
                    width: new_width,
                    height: new_height,
                }));
            }
        }
    }
}

#[tauri::command]
pub fn close_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.close();
    }
}

#[tauri::command]
pub fn close_window_command(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}
