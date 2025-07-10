use chrono::{Local, NaiveTime};
use std::{thread, time::Duration};
// use tauri_plugin_notification::NotificationExt;

#[cfg(target_os = "macos")]
pub fn send_notification(title: &str, message: &str) {
    use mac_notification_sys::*;
    let _ = send_notification(title, None, message, None);
}

// #[cfg(target_os = "windows")]
// pub fn send_notification(title: &str, message: &str) {
//     send_custom_notification(title, message, tauri::AppHandle::default());
// }

// #[tauri::command]
// fn send_custom_notification(title: string, message: string, app: AppHandle) {
//     let _ = app
//         .notification()
//         .builder()
//         .title("Hello from function")
//         .body("This was called outside main()")
//         .show();
// }

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn send_notification(_title: &str, _message: &str) {
    println!("🔔 Notifications not supported on this platform.");
}

pub fn start_timer(minutes: u64) {
    println!("⏳ Starting timer for {} minute(s)...", minutes);
    thread::sleep(Duration::from_secs(minutes * 60));
    println!("⏰ Time's up!");
    // send_notification("Timer", "Time's up!");
}

pub fn set_alarm(target_time: NaiveTime) {
    println!("🔔 Alarm set for {}", target_time);
    loop {
        let now = Local::now().time();
        if now >= target_time {
            println!("⏰ Alarm triggered at {}", now);
            // send_notification("Alarm", &format!("It's {}", now.format("%H:%M")));
            break;
        }
        thread::sleep(Duration::from_secs(30));
    }
}

pub fn run_timer_alarm(command: &str, value: &str) {
    match command {
        "timer" => {
            let minutes: u64 = value.parse().expect("Invalid minutes value");
            start_timer(minutes);
        }
        "alarm" => {
            let target_time = NaiveTime::parse_from_str(value, "%H:%M")
                .expect("Invalid time format, expected HH:MM");
            set_alarm(target_time);
        }
        _ => panic!("Unknown command: {}", command),
    }
}

#[tauri::command]
pub fn run_timer(value: &str) {
    run_timer_alarm("timer", value);
}
#[tauri::command]
pub fn run_alarm(value: &str) {
    run_timer_alarm("alarm", value);
}
