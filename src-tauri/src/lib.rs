use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use std::fs;
use lazy_static::lazy_static;
use screenshots::Screen;
use tauri::Emitter;
use tokio::io::AsyncWriteExt;

#[derive(Clone, PartialEq)]
enum TaskStatus {
    Active,
    Stopping,
    Stopped,
}

// Global state to track running screenshot tasks
lazy_static! {
    static ref RUNNING_TASKS: Arc<Mutex<HashMap<String, TaskStatus>>> = Arc::new(Mutex::new(HashMap::new()));
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn start_screenshotting(window: tauri::Window) -> Result<String, String> {
    // Clean up inactive tasks by removing entries with Stopped status
    {
        let mut tasks = RUNNING_TASKS.lock().map_err(|e| e.to_string())?;
        tasks.retain(|_id, status| match status {
            TaskStatus::Stopped => false,  // Remove stopped tasks
            _ => true,  // Keep active and stopping tasks
        });
    }

    // Check if there are still any active tasks running
    {
        let tasks = RUNNING_TASKS.lock().map_err(|e| e.to_string())?;
        let has_active_task = tasks.values().any(|status| match status {
            TaskStatus::Active | TaskStatus::Stopping => true,
            TaskStatus::Stopped => false,
        });

        if has_active_task {
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
        tasks.insert(session_id.clone(), TaskStatus::Active);
    }

    let session_id_clone = session_id.clone();

    // Start scheduled screenshotting in a background task
    tokio::spawn(async move {
        let start_time = Instant::now();

        loop {
            // Check if stop was requested before taking a screenshot
            let should_continue = {
                let tasks = RUNNING_TASKS.lock().unwrap();
                match tasks.get(&session_id_clone) {
                    Some(TaskStatus::Active) => true,
                    _ => false,
                }
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
                    match tasks.get(&session_id_clone) {
                        Some(TaskStatus::Active) => true,
                        _ => false,
                    }
                };

                if !should_continue {
                    break;
                }
            }
        }

        // Notify completion when stopped
        window.emit("screenshotting-finished", format!("Screenshotting stopped for session: {}", session_id_clone)).unwrap();

        // Update the task status to stopped
        {
            let mut tasks = RUNNING_TASKS.lock().unwrap();
            tasks.insert(session_id_clone, TaskStatus::Stopped);
        }
    });

    Ok(format!("Started screenshotting session: {} (screenshots will be taken every 15 minutes)", session_id))
}

#[tauri::command]
fn stop_screenshotting() -> Result<String, String> {
    let tasks = RUNNING_TASKS.lock().map_err(|e| e.to_string())?;
    // Mark all active tasks as stopping (this will cause them to stop on next check)
    // We need to get the session IDs first, then update them, to avoid borrow checker issues
    let session_ids: Vec<String> = tasks.keys().cloned().collect();

    drop(tasks); // Explicitly drop the immutable lock

    // Now get a mutable lock to update all entries
    let mut tasks = RUNNING_TASKS.lock().map_err(|e| e.to_string())?;
    for session_id in &session_ids {
        if let Some(status) = tasks.get_mut(session_id) {
            if *status == TaskStatus::Active {
                *status = TaskStatus::Stopping;
            }
        }
    }

    Ok("Stop signal sent to all screenshotting sessions".to_string())
}

// Global state to track recording processes
use std::process::{Child, Command};
lazy_static! {
    static ref RECORDING_PROCESSES: Arc<Mutex<HashMap<String, Child>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[tauri::command]
async fn start_recording(window: tauri::Window) -> Result<String, String> {
    // Check if there's already a recording in progress
    {
        let processes = RECORDING_PROCESSES.lock().map_err(|e| e.to_string())?;
        if !processes.is_empty() {
            return Err("A recording session is already in progress".to_string());
        }
        drop(processes);
    }

    // Create recordings directory
    let mut dir = std::env::current_dir().map_err(|e| e.to_string())?;
    dir.push("recordings");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // Create a unique session ID
    let session_id = uuid::Uuid::new_v4().to_string();
    let session_id_clone = session_id.clone();
    let video_path = dir.join(format!("recording_{}.mkv", session_id));
    let video_path_str = video_path.to_string_lossy().to_string();

    // Look for bundled FFmpeg first, then try system FFmpeg
    let ffmpeg_path = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("ffmpeg.exe"); // On Windows; would be "ffmpeg" on other platforms

    let ffmpeg_cmd = if ffmpeg_path.exists() {
        ffmpeg_path.to_string_lossy().to_string()
    } else {
        // Check if system FFmpeg is available
        match std::process::Command::new("ffmpeg").arg("-version").output() {
            Ok(_) => "ffmpeg".to_string(), // System FFmpeg is available
            Err(_) => {
                // Neither bundled nor system FFmpeg found, attempt to download
                window.emit("recording-progress", "FFmpeg not found, downloading...").unwrap();

                // Download FFmpeg automatically
                if let Err(e) = download_ffmpeg_bundled(window.clone(), &ffmpeg_path).await {
                    eprintln!("Failed to download FFmpeg: {}", e);
                    return Err("FFmpeg is required for recording but could not be downloaded".to_string());
                } else {
                    window.emit("recording-progress", "FFmpeg downloaded successfully!").unwrap();
                    ffmpeg_path.to_string_lossy().to_string()
                }
            }
        }
    };

    // Try to start FFmpeg process for direct screen recording
    let mut child = Command::new(&ffmpeg_cmd)
        .args(&[
            "-f", "gdigrab",  // On Windows, use gdigrab for screen capture
            "-i", "desktop",  // Capture the entire desktop
            "-vcodec", "libx264",  // Use H.264 codec
            "-crf", "28",  // Slightly lower quality for better compatibility
            "-preset", "ultrafast",  // Use ultrafast for real-time encoding
            "-pix_fmt", "yuv420p",   // Ensure maximum compatibility
            "-y",  // Overwrite output file
            &video_path_str
        ])
        .spawn()
        .map_err(|e| format!("Failed to start FFmpeg for recording: {}", e))?;

    // Store the process
    {
        let mut processes = RECORDING_PROCESSES.lock().map_err(|e| e.to_string())?;
        processes.insert(session_id.clone(), child);
    }

    window.emit("recording-started", format!("Started recording session: {}", session_id)).unwrap();

    // Spawn a task to monitor the recording process
    tokio::spawn(async move {
        // Simulate recording progress updates
        let mut counter = 0;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            counter += 5;
            window.emit("recording-progress", format!("Recording in progress... {}s", counter)).unwrap();

            // Check if the process is still running or if it should stop
            let should_continue = {
                let processes = RECORDING_PROCESSES.lock().unwrap();
                processes.contains_key(&session_id)
            };

            if !should_continue {
                break; // Stop monitoring if process has been stopped
            }
        }
    });

    Ok(format!("Recording started with session ID: {}", session_id_clone))
}

async fn download_ffmpeg_bundled(window: tauri::Window, ffmpeg_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::fs::File;
    use futures_util::StreamExt;

    // Determine the appropriate FFmpeg build based on the platform
    let (download_url, executable_name) = {
        #[cfg(target_os = "windows")]
        {
            ("https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip", "ffmpeg.exe")
        }
        #[cfg(target_os = "macos")]
        {
            // For macOS, we would need a different URL
            return Err("macOS automatic FFmpeg download not implemented".into());
        }
        #[cfg(target_os = "linux")]
        {
            // For Linux, we would need a different URL
            return Err("Linux automatic FFmpeg download not implemented".into());
        }
    };

    // Create HTTP client and initiate download
    println!("Downloading FFmpeg from: {}", download_url);

    let client = reqwest::Client::new();
    let response = client.get(download_url).send().await?;
    let total_size = response.content_length().unwrap_or(0);

    if total_size > 0 {
        window.emit("recording-progress", format!("Starting FFmpeg download ({:.2} MB)...", total_size as f64 / (1024.0 * 1024.0))).unwrap();
    }

    // Create a temporary file to save the download
    let temp_zip_path = ffmpeg_path.parent().unwrap().join("ffmpeg_temp.zip");
    let mut temp_file = tokio::fs::File::create(&temp_zip_path).await?;

    // Stream the download with progress tracking
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        temp_file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = (downloaded as f64 / total_size as f64) * 100.0;
            window.emit("recording-progress", format!("Downloading FFmpeg: {:.1}%...", progress)).unwrap();
        }
    }

    temp_file.flush().await?;
    drop(temp_file); // Close the file before processing

    // Extract the ZIP file
    let zip_file = std::fs::File::open(&temp_zip_path)?;
    let mut archive = zip::ZipArchive::new(zip_file)?;

    // Look for the executable in the archive
    let mut found_executable = false;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let filename = file.name().to_lowercase();

        // Look for the executable file
        if filename.ends_with(executable_name) {
            // Extract this specific file to the target location
            let mut output_file = File::create(ffmpeg_path)?;
            std::io::copy(&mut file, &mut output_file)?;
            output_file.sync_all()?;

            // Make it executable on Unix systems (not needed on Windows)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(ffmpeg_path, std::fs::Permissions::from_mode(0o755))?;
            }

            found_executable = true;
            break;
        }
    }

    // Delete the temporary ZIP file
    std::fs::remove_file(&temp_zip_path)?;

    if found_executable {
        Ok(())
    } else {
        Err(format!("{} not found in the downloaded archive", executable_name).into())
    }
}

#[tauri::command]
async fn stop_recording(window: tauri::Window) -> Result<String, String> {
    let mut processes = RECORDING_PROCESSES.lock().map_err(|e| e.to_string())?;

    if processes.is_empty() {
        return Ok("No recording in progress".to_string());
    }

    // Terminate all recording processes
    let session_ids: Vec<String> = processes.keys().cloned().collect();
    for session_id in &session_ids {
        if let Some(mut child) = processes.get_mut(session_id) {
            // On Windows, we have to kill the process as there's no graceful way to stop FFmpeg
            let _ = child.kill(); // Force kill the process
        }
    }

    // Clear the process map
    processes.clear();

    // Update the UI
    window.emit("recording-finished", "Recording stopped. Video file is being finalized, please wait a few seconds before opening.").unwrap();

    Ok("Recording stopped. Video file is being finalized, please wait a few seconds before opening.".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, start_screenshotting, stop_screenshotting, start_recording, stop_recording])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
