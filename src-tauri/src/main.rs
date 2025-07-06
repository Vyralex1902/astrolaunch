#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tauri::{Manager, Window};

use std::path::Path;
use strsim;
use walkdir::WalkDir;

use arboard::Clipboard;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::{thread, time::Duration};

#[tauri::command]
fn get_snippets() -> Result<Vec<(String, String)>, String> {
    use std::fs;
    use std::io::Read;
    use std::path::PathBuf;

    let snippet_dir = PathBuf::from("snippets");
    let mut snippets = Vec::new();

    if snippet_dir.exists() && snippet_dir.is_dir() {
        for entry in fs::read_dir(snippet_dir).map_err(|e| e.to_string())? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                        let mut file = fs::File::open(&path).map_err(|e| e.to_string())?;
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)
                            .map_err(|e| e.to_string())?;
                        snippets.push((name.to_string(), contents));
                    }
                }
            }
        }
    }

    Ok(snippets)
}

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
fn search_files(query: &str) -> Result<Vec<String>, String> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    use std::fs;

    #[cfg(target_os = "macos")]
    let search_paths = vec![Path::new("/")];

    #[cfg(target_os = "windows")]
    let search_paths = vec![Path::new("C:\\"), Path::new("D:\\"), Path::new("E:\\")];

    let mut heap = BinaryHeap::new();
    let mut seen = std::collections::HashSet::new();

    for root in search_paths {
        let walker = WalkDir::new(root).into_iter();
        for entry in walker.filter_map(|e| e.ok()).take(100_000) {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = Path::new(path).file_name().and_then(|n| n.to_str()) {
                    let score = strsim::jaro_winkler(query, name);
                    if score > 0.6 && seen.insert(path.to_path_buf()) {
                        heap.push(Reverse((
                            (score * 10000.0) as i32,
                            path.display().to_string(),
                        )));
                        if heap.len() > 8 {
                            heap.pop();
                        }
                    }
                }
            }
        }
    }

    let mut results: Vec<_> = heap
        .into_sorted_vec()
        .into_iter()
        .map(|Reverse((_, path))| path)
        .collect();

    results.reverse(); // highest score first
    Ok(results)
}

#[tauri::command]
fn set_volume(volume: u8) -> Result<(), String> {
    if volume > 100 {
        return Err("Volume must be between 0 and 100".into());
    }

    #[cfg(target_os = "macos")]
    {
        let script = format!("set volume output volume {}", volume);
        let status = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .status()
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("Failed to set volume on macOS".into())
        }
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("nircmd.exe")
            .args(&["setsysvolume", &(volume as u32 * 65535 / 100).to_string()])
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            _ => Err("Failed to set volume on Windows; please install nircmd.exe".into()),
        }
    }
}

#[tauri::command]
fn mute_volume() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let script = format!("set volume output volume {}", 0);
        let status = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .status()
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("Failed to set volume on macOS".into())
        }
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("nircmd.exe")
            .args(&["setsysvolume", &(0 as u32 * 65535 / 100).to_string()])
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            _ => Err("Failed to set volume on Windows; please install nircmd.exe".into()),
        }
    }
}

#[tauri::command]
fn increase_volume(delta: u8) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .arg("-e")
            .arg("output volume of (get volume settings)")
            .output()
            .map_err(|e| e.to_string())?;

        let current_vol_str = String::from_utf8_lossy(&output.stdout);
        let current_vol: u8 = current_vol_str.trim().parse().unwrap_or(50);
        let new_vol = (current_vol.saturating_add(delta)).min(100);

        set_volume(new_vol)
    }

    #[cfg(target_os = "windows")]
    {
        let delta_value = delta as u32 * 65535 / 100;
        let status = Command::new("nircmd.exe")
            .args(&["changesysvolume", &delta_value.to_string()])
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            _ => Err("Failed to increase volume on Windows; please install nircmd.exe".into()),
        }
    }
}

#[tauri::command]
fn decrease_volume(delta: u8) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .arg("-e")
            .arg("output volume of (get volume settings)")
            .output()
            .map_err(|e| e.to_string())?;

        let current_vol_str = String::from_utf8_lossy(&output.stdout);
        let current_vol: u8 = current_vol_str.trim().parse().unwrap_or(50);
        let new_vol = current_vol.saturating_sub(delta);

        set_volume(new_vol)
    }

    #[cfg(target_os = "windows")]
    {
        let delta_value = delta as u32 * 65535 / 100;
        let neg_delta = (0u32).wrapping_sub(delta_value);

        let status = Command::new("nircmd.exe")
            .args(&["changesysvolume", &neg_delta.to_string()])
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            _ => Err("Failed to decrease volume on Windows; please install nircmd.exe".into()),
        }
    }
}

#[tauri::command]
fn set_brightness(brightness: u8) -> Result<(), String> {
    if brightness > 100 {
        return Err("Brightness must be between 0 and 100".into());
    }

    #[cfg(target_os = "macos")]
    {
        let level = brightness as f64 / 100.0;
        let status = Command::new("brightness")
            .arg(&level.to_string())
            .status()
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("Failed to set brightness on macOS".into())
        }
    }

    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "powershell (Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods).WmiSetBrightness(1,{})",
            brightness
        );

        let status = Command::new("powershell")
            .args(&["-Command", &script])
            .status()
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("Failed to set brightness on Windows".into())
        }
    }
}

#[tauri::command]
fn increase_brightness(delta: u8) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("brightness")
            .arg("-l")
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let current_level = stdout
            .lines()
            .find_map(|line| {
                if line.contains("brightness") {
                    line.split_whitespace()
                        .last()
                        .and_then(|v| v.parse::<f64>().ok())
                } else {
                    None
                }
            })
            .unwrap_or(0.5);

        let new_level = (current_level + (delta as f64 / 100.0)).min(1.0);
        set_brightness((new_level * 100.0) as u8)
    }

    #[cfg(target_os = "windows")]
    {
        Err("Incremental brightness on Windows not implemented".into())
    }
}

#[tauri::command]
fn decrease_brightness(delta: u8) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("brightness")
            .arg("-l")
            .output()
            .map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let current_level = stdout
            .lines()
            .find_map(|line| {
                if line.contains("brightness") {
                    line.split_whitespace()
                        .last()
                        .and_then(|v| v.parse::<f64>().ok())
                } else {
                    None
                }
            })
            .unwrap_or(0.5);

        let new_level = (current_level - (delta as f64 / 100.0)).max(0.0);
        set_brightness((new_level * 100.0) as u8)
    }

    #[cfg(target_os = "windows")]
    {
        Err("Decremental brightness on Windows not implemented".into())
    }
}

#[tauri::command]
fn media_play() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let script = r#"tell application "Music" to play"#;
        let status = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .status()
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("Failed to play media on macOS".into())
        }
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("nircmd.exe")
            .arg("sendkeypress")
            .arg("media_play_pause")
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            _ => Err("Failed to play media on Windows; please install nircmd.exe".into()),
        }
    }
}

#[tauri::command]
fn media_pause() -> Result<(), String> {
    media_play()
}

#[tauri::command]
fn media_skip() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let script = r#"tell application "Music" to next track"#;
        let status = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .status()
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("Failed to skip media on macOS".into())
        }
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("nircmd.exe")
            .arg("sendkeypress")
            .arg("media_next")
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            _ => Err("Failed to skip media on Windows; please install nircmd.exe".into()),
        }
    }
}

#[tauri::command]
fn media_previous() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let script = r#"tell application "Music" to previous track"#;
        let status = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .status()
            .map_err(|e| e.to_string())?;

        if status.success() {
            Ok(())
        } else {
            Err("Failed to go to previous media track on macOS".into())
        }
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("nircmd.exe")
            .arg("sendkeypress")
            .arg("media_prev")
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            _ => Err(
                "Failed to go to previous media track on Windows; please install nircmd.exe".into(),
            ),
        }
    }
}

#[tauri::command]
fn minimize_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
    }
}

#[tauri::command]
fn maximize_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.maximize();
    }
}

#[tauri::command]
fn resize_window_80(app: tauri::AppHandle) {
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
fn close_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.close();
    }
}

#[tauri::command]
fn close_window_command(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
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

// Command to launch an app by name or path
#[tauri::command]
fn launch_app(app_name: &str) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("open")
            .args(&["-a", app_name])
            .output()
            .map_err(|e| format!("Failed to execute: {}", e))?;

        if output.status.success() {
            Ok(format!("Launched {}", app_name))
        } else {
            Err(format!(
                "Failed to launch {}: {}",
                app_name,
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("cmd")
            .args(&["/C", "start", "", app_name])
            .output()
            .map_err(|e| format!("Failed to execute: {}", e))?;

        if output.status.success() {
            Ok(format!("Launched {}", app_name))
        } else {
            Err(format!(
                "Failed to launch {}: {}",
                app_name,
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("Unsupported OS".into())
    }
}

#[derive(Serialize)]
struct AppInfo {
    name: String,
    path: String,
}

#[tauri::command]
fn list_apps() -> Result<Vec<AppInfo>, String> {
    #[cfg(target_os = "macos")]
    {
        let app_dirs = vec![
            PathBuf::from("/Applications"),
            PathBuf::from("/System/Applications"),
        ];
        let mut apps = Vec::new();
        let mut seen_paths = std::collections::HashSet::new();

        for dir in app_dirs {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "app" {
                            let path_str = path.display().to_string();
                            if seen_paths.insert(path_str.clone()) {
                                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                                    apps.push(AppInfo {
                                        name: name.to_string(),
                                        path: path_str,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(apps)
    }

    #[cfg(target_os = "windows")]
    {
        let start_menu = PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs");
        let mut apps = Vec::new();
        if let Ok(entries) = fs::read_dir(start_menu) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "lnk" {
                        if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                            apps.push(AppInfo {
                                name: name.to_string(),
                                path: path.display().to_string(),
                            });
                        }
                    }
                }
            }
        }
        Ok(apps)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    Err("Unsupported OS".to_string())
}

fn show_and_focus_window(window: &Window) {
    if let Ok(is_minimized) = window.is_minimized() {
        if is_minimized {
            let _ = window.unminimize();
        }
    }
    let _ = window.show();
    let _ = window.set_focus();
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
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            launch_app,
            list_apps,
            minimize_window,
            maximize_window,
            resize_window_80,
            close_window,
            calculate_expression,
            search_web,
            close_window_command,
            set_volume,
            mute_volume,
            increase_volume,
            decrease_volume,
            set_brightness,
            increase_brightness,
            decrease_brightness,
            media_play,
            media_pause,
            media_skip,
            media_previous,
            search_files,
            record_clipboard,
            get_clipboard_history,
            clear_clipboard_history,
            open_link,
            get_snippets,
            translate_sentence
        ])
        .setup(move |app| {
            // Set activation poicy to Accessory to prevent the app icon from showing on the dock
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running");
}
