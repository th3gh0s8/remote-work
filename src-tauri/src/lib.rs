use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time::{sleep, Duration, Instant};
use std::fs;
use lazy_static::lazy_static;
use screenshots::Screen;
use tauri::Emitter;

// Global state to track screenshot sessions
lazy_static! {
    static ref SCREENSHOT_SESSIONS: Arc<Mutex<HashMap<String, bool>>> = Arc::new(Mutex::new(HashMap::new()));
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn start_screenshotting(window: tauri::Window) -> Result<String, String> {
    // Create a unique session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    // Create screenshots directory
    let mut dir = std::env::current_dir().map_err(|e| e.to_string())?;
    dir.push("screenshots");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // Store session state
    {
        let mut sessions = SCREENSHOT_SESSIONS.lock().map_err(|e| e.to_string())?;
        sessions.insert(session_id.clone(), true);
    }

    let session_id_clone = session_id.clone();

    // Start scheduled screenshotting in a background task
    tokio::spawn(async move {
        let start_time = Instant::now();

        while {
            // Check if screenshotting should continue
            let sessions = SCREENSHOT_SESSIONS.lock().unwrap();
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
                                let filename = format!("screenshot_{}_{}.png", session_id_clone, timestamp);
                                let filepath = dir.join(&filename);

                                if let Err(e) = img.save(&filepath) {
                                    eprintln!("Failed to save screenshot: {}", e);
                                } else {
                                    // Notify that screenshot was taken
                                    window.emit("screenshot-taken", format!("Screenshot saved: {}", filename)).unwrap();
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to capture screenshot: {}", e);
                            }
                        }
                    } else {
                        eprintln!("No screens found");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get screens: {}", e);
                }
            }

            // Wait for 15 minutes (900,000 milliseconds) before taking the next screenshot
            sleep(Duration::from_secs(15 * 60)).await;
        }

        // Notify completion when stopped
        window.emit("screenshotting-finished", format!("Screenshotting stopped for session: {}", session_id_clone)).unwrap();
    });

    Ok(format!("Started screenshotting session: {} (screenshots will be taken every 15 minutes)", session_id))
}

#[tauri::command]
fn stop_screenshotting() -> Result<String, String> {
    let mut sessions = SCREENSHOT_SESSIONS.lock().map_err(|e| e.to_string())?;
    for (_id, active) in sessions.iter_mut() {
        if *active {
            *active = false;
        }
    }
    Ok("Stopped all screenshotting sessions".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start_screenshotting, stop_screenshotting])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
