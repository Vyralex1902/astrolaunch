use chrono::Local;

#[tauri::command]
pub fn get_current_time() -> String {
    Local::now().format("%d %b %Y %H:%M").to_string()
}
