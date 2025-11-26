use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use std::fs;
use lazy_static::lazy_static;
use screenshots::Screen;
use tauri::Emitter;

// Global state to track running screenshot tasks
lazy_static! {
    static ref RUNNING_TASKS: Arc<Mutex<HashMap<String, bool>>> = Arc::new(Mutex::new(HashMap::new()));
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn start_screenshotting(window: tauri::Window) -> Result<String, String> {
    // Clean up inactive tasks by removing entries with false value
    {
        let mut tasks = RUNNING_TASKS.lock().map_err(|e| e.to_string())?;
        tasks.retain(|_id, &mut active| active); // Keep only active tasks
    }

    // Check if there are still any active tasks running
    {
        let tasks = RUNNING_TASKS.lock().map_err(|e| e.to_string())?;
        if !tasks.is_empty() {
            return Err("A screenshotting session is already running".to_string());
        }
        drop(tasks);
    }

    // Create a unique session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    // Create screenshots directory
    let mut dir = std::env::current_dir().map_err(|e| e.to_string())?;
    dir.push("screenshots");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // Store task state as active
    {
        let mut tasks = RUNNING_TASKS.lock().map_err(|e| e.to_string())?;
        tasks.insert(session_id.clone(), true);
    }

    let session_id_clone = session_id.clone();

    // Start scheduled screenshotting in a background task
    tokio::spawn(async move {
        let start_time = Instant::now();

        loop {
            // Check if stop was requested before taking a screenshot
            let should_continue = {
                let tasks = RUNNING_TASKS.lock().unwrap();
                *tasks.get(&session_id_clone).unwrap_or(&false)
            };

            if !should_continue {
                break;
            }

            // Take screenshot
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

            // Wait for 15 minutes before taking the next screenshot, but check for stop signal
            // Wait in 1-second intervals to check the stop flag
            for _ in 0..(15 * 60) {
                tokio::time::sleep(Duration::from_secs(1)).await;

                // Check if stop was requested
                let should_continue = {
                    let tasks = RUNNING_TASKS.lock().unwrap();
                    *tasks.get(&session_id_clone).unwrap_or(&false)
                };

                if !should_continue {
                    break;
                }
            }
        }

        // Notify completion when stopped
        window.emit("screenshotting-finished", format!("Screenshotting stopped for session: {}", session_id_clone)).unwrap();

        // Remove the task from the running tasks list
        {
            let mut tasks = RUNNING_TASKS.lock().unwrap();
            tasks.remove(&session_id_clone);
        }
    });

    Ok(format!("Started screenshotting session: {} (screenshots will be taken every 15 minutes)", session_id))
}

#[tauri::command]
fn stop_screenshotting() -> Result<String, String> {
    let mut tasks = RUNNING_TASKS.lock().map_err(|e| e.to_string())?;
    // Mark all tasks as inactive (this will cause them to stop on next check)
    for (_id, active) in tasks.iter_mut() {
        *active = false;
    }
    Ok("Stop signal sent to all screenshotting sessions".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start_screenshotting, stop_screenshotting])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
