#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Deserialize;
use serde::Serialize;

use arboard::Clipboard;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::{thread, time::Duration};
use tauri_plugin_notification::NotificationExt;

mod appsLib;
mod clockLib;
mod liveDataLib;
mod mediaLib;
mod searchFilesLib;
mod settings;
mod snippetsLib;
mod systemManagementLib;
mod windowMngLib;

#[tauri::command]
fn open_link(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/C", "start", "", url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

// Clipboard history (text only)
struct ClipboardHistory {
    items: Vec<String>,
}

impl ClipboardHistory {
    fn new() -> Self {
        ClipboardHistory { items: Vec::new() }
    }

    fn add_text(&mut self, text: String) {
        if self.items.last().map_or(true, |last| last != &text) {
            self.items.push(text);
            if self.items.len() > 10 {
                self.items.remove(0);
            }
        }
    }

    fn clear(&mut self) {
        self.items.clear();
    }
}

static CLIPBOARD_HISTORY: Lazy<Mutex<ClipboardHistory>> =
    Lazy::new(|| Mutex::new(ClipboardHistory::new()));

#[tauri::command]
fn record_clipboard() -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    let mut history = CLIPBOARD_HISTORY.lock().unwrap();

    if let Ok(text) = clipboard.get_text() {
        if !text.trim().is_empty() {
            history.add_text(text);
            return Ok(());
        }
    }

    Err("Clipboard empty or unsupported format".into())
}

#[tauri::command]
fn get_clipboard_history() -> Vec<String> {
    let history = CLIPBOARD_HISTORY.lock().unwrap();
    history.items.clone()
}

#[tauri::command]
fn clear_clipboard_history() {
    let mut history = CLIPBOARD_HISTORY.lock().unwrap();
    history.clear();
}

#[tauri::command]
fn calculate_expression(expression: &str) -> Result<f64, String> {
    meval::eval_str(expression).map_err(|e| e.to_string())
}

#[tauri::command]
fn search_web(query: &str) -> Result<(), String> {
    use std::process::Command;
    use urlencoding::encode;

    let url = if query.starts_with("http") {
        query.to_string()
    } else {
        format!("https://www.google.com/search?q={}", encode(query))
    };

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(&["/C", "start", "", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Serialize)]
struct AppInfo {
    name: String,
    path: String,
}

#[derive(Deserialize)]
struct TranslationResponse {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

#[tauri::command]
async fn translate_sentence(
    query: String,
    lang_from: String,
    lang_to: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = "https://translate.googleapis.com/translate_a/single";

    let params = [
        ("client", "gtx"),
        ("sl", &lang_from),
        ("tl", &lang_to),
        ("dt", "t"),
        ("q", &query),
    ];

    let res = client
        .get(url)
        .query(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

    let translated = json[0][0][0]
        .as_str()
        .ok_or("Failed to extract translation")?
        .to_string();

    Ok(translated)
}

#[tauri::command]
fn settings_get_autostart(app: tauri::AppHandle) -> bool {
    settings::is_autostart_enabled(&app)
}

#[tauri::command]
fn settings_toggle_autostart(app: tauri::AppHandle) -> Result<(), String> {
    if settings::is_autostart_enabled(&app) {
        settings::disable_autostart(&app);
    } else {
        settings::enable_autostart(&app);
    }
    Ok(())
}

#[tauri::command]
fn run_macos_shortcut(name: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("shortcuts")
            .args(&["run", name])
            .output()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err("This command only works on macOS.".into())
    }
}

fn main() {
    // Spawn clipboard polling thread to track clipboard changes automatically
    let clipboard_history = &CLIPBOARD_HISTORY;
    std::thread::spawn(move || {
        let mut last_text = String::new();
        let mut clipboard = match Clipboard::new() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to initialize clipboard for polling: {}", e);
                return;
            }
        };

        loop {
            std::thread::sleep(Duration::from_millis(500));

            let current_text = clipboard.get_text();
            match current_text {
                Ok(text) => {
                    if !text.trim().is_empty() && text != last_text {
                        let mut history = clipboard_history.lock().unwrap();
                        history.add_text(text.clone());
                        last_text = text;
                    }
                }
                Err(_) => {
                    // Clipboard read failed, ignore and retry
                    continue;
                }
            }
        }
    });

    use tauri_plugin_autostart::MacosLauncher;
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            calculate_expression,
            search_web,
            appsLib::launch_app,
            appsLib::list_apps,
            windowMngLib::minimize_window,
            windowMngLib::maximize_window,
            windowMngLib::resize_window_80,
            windowMngLib::close_window,
            windowMngLib::close_window_command,
            systemManagementLib::set_brightness,
            systemManagementLib::increase_brightness,
            systemManagementLib::decrease_brightness,
            systemManagementLib::restart_system,
            systemManagementLib::shutdown_system,
            systemManagementLib::lock_system,
            systemManagementLib::empty_trash,
            mediaLib::set_volume,
            mediaLib::mute_volume,
            mediaLib::increase_volume,
            mediaLib::decrease_volume,
            mediaLib::media_play,
            mediaLib::media_pause,
            mediaLib::media_skip,
            mediaLib::media_previous,
            searchFilesLib::search_files,
            snippetsLib::get_snippets,
            record_clipboard,
            get_clipboard_history,
            clear_clipboard_history,
            open_link,
            translate_sentence,
            liveDataLib::get_current_time,
            clockLib::run_timer,
            clockLib::run_alarm,
            settings_get_autostart,
            settings_toggle_autostart,
            run_macos_shortcut,
        ])
        .setup(move |app| {
            app.notification()
                .builder()
                .title("AstroLaunch On")
                .body("AstroLaunch is now running!")
                .show()
                .unwrap();

            let app_handle = app.handle();
            windowMngLib::show_and_center_window(app_handle.clone());

            // let win = app.get_window("main").unwrap();

            // #[cfg(target_os = "macos")]
            // win.set_decorations(false)?;

            // // Remove shadow (macOS-specific)
            // #[cfg(target_os = "macos")]
            // win.set_shadow(false)?;

            // Set activation poicy to Accessory to prevent the app icon from showing on the dock
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }
            Ok(())
        })
        .plugin(tauri_plugin_notification::init())
        .run(tauri::generate_context!())
        .expect("error while running");
}
