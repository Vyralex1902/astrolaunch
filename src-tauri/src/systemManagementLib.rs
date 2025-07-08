#[tauri::command]
pub fn set_brightness(brightness: u8) -> Result<(), String> {
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
pub fn increase_brightness(delta: u8) -> Result<(), String> {
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
pub fn decrease_brightness(delta: u8) -> Result<(), String> {
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
