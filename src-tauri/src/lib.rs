use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use std::fs;
use lazy_static::lazy_static;
use screenshots::Screen;
use tauri::Emitter;

// Global state to track recording sessions
lazy_static! {
    static ref RECORDING_SESSIONS: Arc<Mutex<HashMap<String, bool>>> = Arc::new(Mutex::new(HashMap::new()));
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn start_recording(window: tauri::Window) -> Result<String, String> {

    // Create a unique session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    // Create screenshots directory
    let mut dir = std::env::current_dir().map_err(|e| e.to_string())?;
    dir.push("recordings");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // Store session state
    {
        let mut sessions = RECORDING_SESSIONS.lock().map_err(|e| e.to_string())?;
        sessions.insert(session_id.clone(), true);
    }

    let session_id_clone = session_id.clone();

    // Start recording in a background task
    tokio::spawn(async move {
        let mut frame_count = 0;
        let start_time = std::time::Instant::now();

        while {
            // Check if recording should continue
            let sessions = RECORDING_SESSIONS.lock().unwrap();
            let should_continue = *sessions.get(&session_id_clone).unwrap_or(&false);
            drop(sessions);
            should_continue
        } {
            match Screen::all() {
                Ok(screens) => {
                    if let Some(primary_screen) = screens.first() {
                        match primary_screen.capture_area(0, 0, primary_screen.display_info.width, primary_screen.display_info.height) {
                            Ok(img) => {
                                let timestamp = start_time.elapsed().as_millis();
                                let filename = format!("frame_{}_{}.png", session_id_clone, timestamp);
                                let filepath = dir.join(&filename);

                                if let Err(e) = img.save(&filepath) {
                                    eprintln!("Failed to save frame: {}", e);
                                    break;
                                }

                                frame_count += 1;
                            }
                            Err(e) => {
                                eprintln!("Failed to capture frame: {}", e);
                                break;
                            }
                        }
                    } else {
                        eprintln!("No screens found");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get screens: {}", e);
                    break;
                }
            }

            // Capture at ~10 FPS (100ms interval)
            sleep(Duration::from_millis(100)).await;
        }

        // Notify completion
        window.emit("recording-finished", format!("Recorded {} frames to recordings directory", frame_count)).unwrap();
    });

    Ok(format!("Started recording session: {}", session_id))
}

#[tauri::command]
fn stop_recording() -> Result<String, String> {
    let mut sessions = RECORDING_SESSIONS.lock().map_err(|e| e.to_string())?;
    for (_id, active) in sessions.iter_mut() {
        if *active {
            *active = false;
        }
    }
    Ok("Stopped all recording sessions".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start_recording, stop_recording])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
