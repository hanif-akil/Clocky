#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use chrono::{Local, Timelike};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppSettings {
    time_format: String,
    theme: String,
    bg_opacity: f64,
    bg_image_path: Option<String>,
    bg_image_data: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            time_format: "12".to_string(),
            theme: "Midnight".to_string(),
            bg_opacity: 0.35,
            bg_image_path: None,
            bg_image_data: None,
        }
    }
}

struct AppState {
    settings: Mutex<AppSettings>,
    stopwatch_running: Mutex<bool>,
    stopwatch_start: Mutex<Option<Instant>>,
    stopwatch_elapsed: Mutex<u64>,
    timer_running: Mutex<bool>,
    timer_duration: Mutex<u64>,
    timer_start: Mutex<Option<Instant>>,
    alarm_hour: Mutex<u32>,
    alarm_minute: Mutex<u32>,
    alarm_active: Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            settings: Mutex::new(AppSettings::default()),
            stopwatch_running: Mutex::new(false),
            stopwatch_start: Mutex::new(None),
            stopwatch_elapsed: Mutex::new(0),
            timer_running: Mutex::new(false),
            timer_duration: Mutex::new(60),
            timer_start: Mutex::new(None),
            alarm_hour: Mutex::new(8),
            alarm_minute: Mutex::new(0),
            alarm_active: Mutex::new(false),
        }
    }
}

fn get_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clocky")
        .join("settings.json")
}

fn load_settings() -> AppSettings {
    let path = get_config_path();
    if path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(settings) = serde_json::from_str(&contents) {
                return settings;
            }
        }
    }
    AppSettings::default()
}

fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_current_time(format: String) -> String {
    let now = Local::now();
    if format == "12" {
        now.format("%I:%M:%S %p").to_string()
    } else {
        now.format("%H:%M:%S").to_string()
    }
}

#[tauri::command]
fn get_current_date() -> String {
    Local::now().format("%A, %B %-d, %Y").to_string()
}

#[tauri::command]
fn get_settings(state: tauri::State<AppState>) -> AppSettings {
    state.settings.lock().unwrap().clone()
}

#[tauri::command]
fn update_settings(settings: AppSettings, state: tauri::State<AppState>) -> Result<(), String> {
    let mut current = state.settings.lock().unwrap();
    *current = settings.clone();
    drop(current);
    save_settings(&settings)
}

#[tauri::command]
fn pick_image() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp"])
        .pick_file()
        .map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
fn load_image_as_base64(path: String) -> Result<String, String> {
    let img = image::open(&path).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    
    let mut buffer = Vec::with_capacity((width * height * 4) as usize);
    for pixel in rgba.pixels() {
        buffer.extend_from_slice(&pixel.0);
    }
    
    Ok(base64::encode(&buffer))
}

#[tauri::command]
fn get_elapsed_stopwatch(state: tauri::State<AppState>) -> u64 {
    let running = *state.stopwatch_running.lock().unwrap();
    let start = *state.stopwatch_start.lock().unwrap();
    let elapsed = *state.stopwatch_elapsed.lock().unwrap();
    
    if running {
        if let Some(start_time) = start {
            elapsed + start_time.elapsed().as_millis() as u64
        } else {
            elapsed
        }
    } else {
        elapsed
    }
}

#[tauri::command]
fn start_stopwatch(state: tauri::State<AppState>) {
    let mut running = state.stopwatch_running.lock().unwrap();
    let mut start = state.stopwatch_start.lock().unwrap();
    
    if !*running {
        *start = Some(Instant::now());
        *running = true;
    }
}

#[tauri::command]
fn pause_stopwatch(state: tauri::State<AppState>) {
    let mut running = state.stopwatch_running.lock().unwrap();
    let mut elapsed = state.stopwatch_elapsed.lock().unwrap();
    let start = *state.stopwatch_start.lock().unwrap();
    
    if *running {
        if let Some(start_time) = start {
            *elapsed += start_time.elapsed().as_millis() as u64;
        }
        *running = false;
    }
}

#[tauri::command]
fn reset_stopwatch(state: tauri::State<AppState>) {
    let mut running = state.stopwatch_running.lock().unwrap();
    let mut elapsed = state.stopwatch_elapsed.lock().unwrap();
    let mut start = state.stopwatch_start.lock().unwrap();
    
    *running = false;
    *elapsed = 0;
    *start = None;
}

#[tauri::command]
fn start_timer(minutes: u32, seconds: u32, state: tauri::State<AppState>) {
    let total_secs = (minutes * 60 + seconds) as u64;
    let mut running = state.timer_running.lock().unwrap();
    let mut duration = state.timer_duration.lock().unwrap();
    let mut start = state.timer_start.lock().unwrap();
    
    *duration = total_secs;
    *start = Some(Instant::now());
    *running = true;
}

#[tauri::command]
fn pause_timer(state: tauri::State<AppState>) {
    let mut running = state.timer_running.lock().unwrap();
    *running = false;
}

#[tauri::command]
fn reset_timer(state: tauri::State<AppState>) {
    let mut running = state.timer_running.lock().unwrap();
    *running = false;
}

#[tauri::command]
fn get_timer_remaining(state: tauri::State<AppState>) -> Option<u64> {
    let running = *state.timer_running.lock().unwrap();
    let duration = *state.timer_duration.lock().unwrap();
    let start = *state.timer_start.lock().unwrap();
    
    if running {
        if let Some(start_time) = start {
            let elapsed = start_time.elapsed().as_secs();
            if elapsed >= duration {
                Some(0)
            } else {
                Some(duration - elapsed)
            }
        } else {
            Some(duration)
        }
    } else {
        Some(duration)
    }
}

#[tauri::command]
fn set_alarm(hour: u32, minute: u32, state: tauri::State<AppState>) {
    let mut alarm_hour = state.alarm_hour.lock().unwrap();
    let mut alarm_minute = state.alarm_minute.lock().unwrap();
    
    *alarm_hour = hour;
    *alarm_minute = minute;
}

#[tauri::command]
fn toggle_alarm(active: bool, state: tauri::State<AppState>) {
    let mut alarm_active = state.alarm_active.lock().unwrap();
    *alarm_active = active;
}

#[tauri::command]
fn check_alarm(state: tauri::State<AppState>) -> bool {
    let alarm_active = *state.alarm_active.lock().unwrap();
    let alarm_hour = *state.alarm_hour.lock().unwrap();
    let alarm_minute = *state.alarm_minute.lock().unwrap();
    
    if !alarm_active {
        return false;
    }
    
    let now = Local::now();
    now.hour() == alarm_hour && now.minute() == alarm_minute && now.second() < 2
}

#[tauri::command]
fn play_alarm_sound() {
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let script = r#"
[console]::beep(880, 180); Start-Sleep -m 120
[console]::beep(880, 180); Start-Sleep -m 120
[console]::beep(1100, 300)
"#;
        let _ = std::process::Command::new("powershell")
            .args(&["-c", script])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("sh")
            .args(&["-c", "afplay /System/Library/Sounds/Ping.aiff; sleep 0.3; afplay /System/Library/Sounds/Ping.aiff; sleep 0.3; afplay /System/Library/Sounds/Ping.aiff"])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("sh")
            .args(&["-c", "canberra-gtk-play -i alarm-clock-elapsed || paplay /usr/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga || aplay /usr/share/sounds/alsa/Front_Center.wav"])
            .spawn();
    }
}

fn main() {
    let settings = load_settings();
    let app_state = AppState {
        settings: Mutex::new(settings),
        ..Default::default()
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_current_time,
            get_current_date,
            get_settings,
            update_settings,
            pick_image,
            load_image_as_base64,
            get_elapsed_stopwatch,
            start_stopwatch,
            pause_stopwatch,
            reset_stopwatch,
            start_timer,
            pause_timer,
            reset_timer,
            get_timer_remaining,
            set_alarm,
            toggle_alarm,
            check_alarm,
            play_alarm_sound
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
