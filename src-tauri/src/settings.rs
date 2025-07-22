use tauri_plugin_autostart::ManagerExt;

pub fn enable_autostart(app: &tauri::AppHandle) {
    let autostart_manager = app.autolaunch();
    if !autostart_manager.is_enabled().unwrap_or(false) {
        autostart_manager.enable().unwrap();
        #[cfg(dev)]
        {
            println!("Autostart enabled");
        }
    }
}
pub fn disable_autostart(app: &tauri::AppHandle) {
    let autostart_manager = app.autolaunch();
    if autostart_manager.is_enabled().unwrap() {
        autostart_manager.disable().unwrap();
        #[cfg(dev)]
        {
            println!("Autostart disabled");
        }
    }
}
pub fn is_autostart_enabled(app: &tauri::AppHandle) -> bool {
    let autostart_manager = app.autolaunch();
    autostart_manager.is_enabled().unwrap_or(false)
}
