use std::process::Command;

#[tauri::command]
pub fn media_play() -> Result<(), String> {
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
pub fn media_pause() -> Result<(), String> {
    media_play()
}

#[tauri::command]
pub fn media_skip() -> Result<(), String> {
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
pub fn media_previous() -> Result<(), String> {
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
pub fn set_volume(volume: u8) -> Result<(), String> {
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
pub fn mute_volume() -> Result<(), String> {
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
pub fn increase_volume(delta: u8) -> Result<(), String> {
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
pub fn decrease_volume(delta: u8) -> Result<(), String> {
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
