use crate::AppInfo;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[tauri::command]
pub fn list_apps() -> Result<Vec<AppInfo>, String> {
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

// Command to launch an app by name or path
#[tauri::command]
pub fn launch_app(app_name: &str) -> Result<String, String> {
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
