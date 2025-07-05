#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use tauri::Manager;
use urlencoding::encode; // Manager trait provides get_window and other window management methods

use serde::Serialize;
use std::fs;
use std::path::PathBuf;

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

// Your existing greet command or remove if you want
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to AstroLaunch 🚀", name)
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            launch_app,
            list_apps,
            minimize_window,
            maximize_window,
            resize_window_80,
            close_window,
            calculate_expression,
            search_web
        ])
        .run(tauri::generate_context!())
        .expect("error while running");
}
