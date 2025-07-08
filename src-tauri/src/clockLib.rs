use chrono::{Local, NaiveTime};
use std::{thread, time::Duration};

#[cfg(target_os = "macos")]
pub fn send_notification(title: &str, message: &str) {
    use mac_notification_sys::*;
    let bundle = get_bundle_identifier_or_default("com.apple.Terminal");
    let _ = send_notification(title, &Some(message.into()), &None, &bundle);
}

#[cfg(target_os = "windows")]
pub fn send_notification(title: &str, message: &str) {
    use windows_notifications::{Toast, ToastBuilder};
    let toast = Toast::new(ToastBuilder::new(title).text1(message));
    let _ = toast.show();
}

#[cfg(target_os = "linux")]
pub fn send_notification(title: &str, message: &str) {
    use notify_rust::Notification;
    let _ = Notification::new().summary(title).body(message).show();
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub fn send_notification(_title: &str, _message: &str) {
    println!("🔔 Notifications not supported on this platform.");
}

pub fn start_timer(minutes: u64) {
    println!("⏳ Starting timer for {} minute(s)...", minutes);
    thread::sleep(Duration::from_secs(minutes * 60));
    println!("⏰ Time's up!");
    send_notification("Timer", "Time's up!");
}

pub fn set_alarm(target_time: NaiveTime) {
    println!("🔔 Alarm set for {}", target_time);
    loop {
        let now = Local::now().time();
        if now >= target_time {
            println!("⏰ Alarm triggered at {}", now);
            send_notification("Alarm", &format!("It's {}", now.format("%H:%M")));
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
pub fn runTimer(value: &str) {
    run_timer_alarm("timer", value);
}
#[tauri::command]
pub fn runAlarm(value: &str) {
    run_timer_alarm("alarm", value);
}
