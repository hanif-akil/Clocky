use slint::*;
use chrono::{Local, Timelike};
use std::time::{Duration, Instant};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use dirs;

#[derive(Serialize, Deserialize, Clone)]
struct BustedSession {
    project_name: String,
    start_time: String,
    end_time: String,
    duration_secs: u64,
}

#[derive(Serialize, Deserialize)]
struct BustedLog {
    sessions: Vec<BustedSession>,
    total_sessions: usize,
    total_duration_secs: u64,
}

impl Default for BustedLog {
    fn default() -> Self {
        Self {
            sessions: Vec::new(),
            total_sessions: 0,
            total_duration_secs: 0,
        }
    }
}

struct BustedState {
    is_running: bool,
    start_time: Option<Instant>,
    elapsed: Duration,
    project_name: String,
    current_session: Option<BustedSession>,
}

impl Default for BustedState {
    fn default() -> Self {
        Self {
            is_running: false,
            start_time: None,
            elapsed: Duration::ZERO,
            project_name: String::new(),
            current_session: None,
        }
    }
}

slint::slint! {
    import { MainWindow } from "ui.slint";
}

fn get_log_file_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Clocky")
        .join("busted");
    
    std::fs::create_dir_all(&config_dir).ok();
    config_dir.join("sessions.json")
}

fn load_busted_log() -> BustedLog {
    let path = get_log_file_path();
    if path.exists() {
        if let Ok(mut file) = File::open(&path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                if let Ok(log) = serde_json::from_str(&contents) {
                    return log;
                }
            }
        }
    }
    BustedLog::default()
}

fn save_busted_log(log: &BustedLog) -> Result<(), String> {
    let path = get_log_file_path();
    let json = serde_json::to_string_pretty(log)
        .map_err(|e| e.to_string())?;
    
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    
    file.write_all(json.as_bytes())
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

fn main() -> Result<(), PlatformError> {
    let main_window = MainWindow::new()?;
    
    // State
    let mut stopwatch_running = false;
    let mut stopwatch_start: Option<Instant> = None;
    let mut stopwatch_elapsed = Duration::ZERO;
    
    let mut timer_running = false;
    let mut timer_duration = Duration::from_secs(60);
    let mut timer_start: Option<Instant> = None;
    let mut timer_left = Duration::from_secs(60);
    
    let mut busted_state = BustedState::default();
    let mut busted_log = load_busted_log();
    
    // Clock update timer
    let clock_weak = main_window.as_weak();
    let _clock_timer = Timer::every(Duration::from_secs(1), move || {
        let Some(main_window) = clock_weak.upgrade() else { return };
        
        let now = Local::now();
        let time_str = now.format("%I:%M:%S %p").to_string();
        let date_str = now.format("%A, %B %-d, %Y").to_string();
        
        // These would be connected to UI properties
        println!("Time: {}, Date: {}", time_str, date_str);
    });
    
    // Stopwatch update timer
    let stopwatch_weak = main_window.as_weak();
    let _stopwatch_timer = Timer::every(Duration::from_millis(10), move || {
        let Some(main_window) = stopwatch_weak.upgrade() else { return };
        
        if stopwatch_running {
            if let Some(start) = stopwatch_start {
                stopwatch_elapsed = start.elapsed();
                let total_secs = stopwatch_elapsed.as_secs();
                let millis = stopwatch_elapsed.subsec_millis() / 10;
                let display = format!("{:02}:{:02}:{:02}.{:02}",
                    total_secs / 3600,
                    (total_secs % 3600) / 60,
                    total_secs % 60,
                    millis
                );
                println!("Stopwatch: {}", display);
            }
        }
    });
    
    // Busted update timer
    let busted_weak = main_window.as_weak();
    let _busted_timer = Timer::every(Duration::from_millis(100), move || {
        let Some(main_window) = busted_weak.upgrade() else { return };
        
        if busted_state.is_running {
            if let Some(start) = busted_state.start_time {
                let current_elapsed = start.elapsed();
                let total_elapsed = busted_state.elapsed + current_elapsed;
                let display = format!("{:02}:{:02}:{:02}",
                    total_elapsed.as_secs() / 3600,
                    (total_elapsed.as_secs() % 3600) / 60,
                    total_elapsed.as_secs() % 60
                );
                println!("Busted Timer: {}", display);
            }
        }
    });
    
    // Event handlers would be connected here
    // For example, start/stop buttons, mode switching, etc.
    
    main_window.run()?;
    Ok(())
}
