use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use std::fs;
use lazy_static::lazy_static;
use screenshots::Screen;
use tauri::Emitter;
use tokio::io::AsyncWriteExt;
use std::time::SystemTime;

// Windows-specific imports
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;


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
                                let img = img;

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

// Global state to track combined recording status
use std::process::{Child, Command};
lazy_static! {
    static ref COMBINED_RECORDING_PROCESS: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
}

#[tauri::command]
async fn start_combined_recording(window: tauri::Window) -> Result<String, String> {
    // Check if there's already a recording in progress
    {
        let process_guard = COMBINED_RECORDING_PROCESS.lock().map_err(|e| e.to_string())?;
        if process_guard.is_some() {
            return Err("A recording session is already in progress".to_string());
        }
        drop(process_guard);
    }

    // Create recordings directory
    let mut dir = std::env::current_dir().map_err(|e| e.to_string())?;
    dir.push("recordings");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    // Create unique session IDs
    let session_id = uuid::Uuid::new_v4().to_string();
    let video_path = dir.join(format!("recording_{}.mkv", session_id));
    let video_path_str = video_path.to_string_lossy().to_string();

    // Look for bundled FFmpeg first
    let ffmpeg_path = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("ffmpeg.exe");

    let ffmpeg_cmd = if ffmpeg_path.exists() {
        ffmpeg_path.to_string_lossy().to_string()
    } else {
        // Check if system FFmpeg is available
        match {
            #[cfg(target_os = "windows")]
            {
                std::process::Command::new("ffmpeg")
                    .arg("-version")
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW flag
                    .output()
            }
            #[cfg(not(target_os = "windows"))]
            {
                std::process::Command::new("ffmpeg")
                    .arg("-version")
                    .output()
            }
        } {
            Ok(_) => "ffmpeg".to_string(),
            Err(_) => {
                // Neither bundled nor system FFmpeg found, attempt to download
                window.emit("recording-progress", "FFmpeg not found, downloading...").unwrap();

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

    // Start the video recording process with FFmpeg
    let child = {
        #[cfg(target_os = "windows")]
        {
            Command::new(&ffmpeg_cmd)
                .args(&[
                    "-f", "gdigrab",
                    "-i", "desktop",
                    "-vcodec", "libx264",
                    "-crf", "28",
                    "-preset", "ultrafast",
                    "-pix_fmt", "yuv420p",
                    "-y",
                    &video_path_str
                ])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW flag
                .spawn()
                .map_err(|e| format!("Failed to start FFmpeg for recording: {}", e))?
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new(&ffmpeg_cmd)
                .args(&[
                    "-f", "gdigrab",
                    "-i", "desktop",
                    "-vcodec", "libx264",
                    "-crf", "28",
                    "-preset", "ultrafast",
                    "-pix_fmt", "yuv420p",
                    "-y",
                    &video_path_str
                ])
                .spawn()
                .map_err(|e| format!("Failed to start FFmpeg for recording: {}", e))?
        }
    };

    // Store the recording process
    {
        let mut process_guard = COMBINED_RECORDING_PROCESS.lock().map_err(|e| e.to_string())?;
        *process_guard = Some(child);
    }

    window.emit("recording-started", format!("Remote Worker: started")).unwrap();

    // Start the screenshot-taking process in parallel
    let screenshot_session_id = session_id.clone();
    let screenshot_window = window.clone();
    tokio::spawn(async move {
        let start_time = Instant::now();

        loop {
            // Check if the recording process is still active
            let is_active = {
                let process_guard = COMBINED_RECORDING_PROCESS.lock().unwrap();
                process_guard.is_some()
            };

            if !is_active {
                break; // Stop if the recording process has been terminated
            }

            // Take a screenshot
            match Screen::all() {
                Ok(screens) => {
                    if let Some(primary_screen) = screens.first() {
                        match primary_screen.capture_area(0, 0, primary_screen.display_info.width, primary_screen.display_info.height) {
                            Ok(img) => {
                                let img = img;

                                // Create screenshots directory
                                let mut screenshots_dir = std::env::current_dir().unwrap();
                                screenshots_dir.push("screenshots");
                                std::fs::create_dir_all(&screenshots_dir).unwrap();

                                let timestamp = start_time.elapsed().as_millis();
                                let filename = format!("snapshot_{}_{}.png", screenshot_session_id, timestamp);
                                let filepath = screenshots_dir.join(&filename);

                                if let Err(e) = img.save(&filepath) {
                                    eprintln!("Failed to save snapshot: {}", e);
                                } else {
                                    screenshot_window.emit("screenshot-taken", format!("Snapshot saved: {}", filename)).unwrap(); // Note: Keeping event name as screenshot-taken for compatibility
                                    // Update user activity since a snapshot was just taken (user is likely active)
                                    if let Ok(mut last_activity) = LAST_USER_ACTIVITY.lock() {
                                        *last_activity = SystemTime::now();
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to capture screenshot: {}", e);
                            }
                        }
                    } else {
                        eprintln!("No screens found for snapshot");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get screens for snapshot: {}", e);
                }
            }

            // Generate a random interval between 5 and 30 minutes in seconds (300 to 1800 seconds)
            let random_interval: u64 = {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                rng.gen_range(300..=1800) // 5 to 30 minutes in seconds
            };

            let screenshot_window_clone = screenshot_window.clone();
            // Wait for the random interval before taking the next screenshot
            // But check every second if recording is still active
            for remaining_seconds in (1..=random_interval).rev() {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                // Emit progress update about the remaining time
                let minutes = remaining_seconds / 60;
                let seconds = remaining_seconds % 60;
                screenshot_window_clone.emit("recording-progress", format!("Next snapshot in: {}m {}s", minutes, seconds)).unwrap();

                let is_active = {
                    let process_guard = COMBINED_RECORDING_PROCESS.lock().unwrap();
                    process_guard.is_some()
                };

                if !is_active {
                    break; // Exit the waiting loop if recording stopped
                }
            }

            // Check again if still active after 15-minute wait
            let is_active = {
                let process_guard = COMBINED_RECORDING_PROCESS.lock().unwrap();
                process_guard.is_some()
            };

            if !is_active {
                break; // Exit the main loop if recording stopped
            }
        }
    });

    Ok(format!("Remote Worker: started: (Session ID: {})", session_id))
}

// Global state to track user activity
lazy_static! {
    static ref LAST_USER_ACTIVITY: Arc<Mutex<SystemTime>> = Arc::new(Mutex::new(SystemTime::now()));
    static ref IDLE_DETECTION_TASK: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(None));
}



use tauri::Manager;

// Function to create an admin window
#[tauri::command]
async fn create_admin_window(window: tauri::Window) -> Result<String, String> {
    let app_handle = window.app_handle();

    // Check if the window already exists
    if app_handle.get_webview_window("admin").is_some() {
        return Ok("Admin window already exists".to_string());
    }

    // Create a new window with the title "Admin"
    let _child_window = tauri::webview::WebviewWindowBuilder::new(
        app_handle,
        "admin",
        tauri::WebviewUrl::App("src/admin.html".into())
    )
    .title("Admin")
    .inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .build()
    .map_err(|e| format!("Failed to create admin window: {}", e))?;

    Ok("Admin window created".to_string())
}

// Internal function to create admin window that can be called from global shortcut
async fn create_admin_window_internal(app_handle: &tauri::AppHandle) -> Result<String, String> {
    // Check if the window already exists
    if app_handle.get_webview_window("admin").is_some() {
        return Ok("Admin window already exists".to_string());
    }

    // Create a new window with the title "Admin"
    let _child_window = tauri::webview::WebviewWindowBuilder::new(
        app_handle,
        "admin",
        tauri::WebviewUrl::App("src/admin.html".into())
    )
    .title("Admin")
    .inner_size(800.0, 600.0)
    .resizable(true)
    .center()
    .build()
    .map_err(|e| format!("Failed to create admin window: {}", e))?;

    Ok("Admin window created".to_string())
}

#[tauri::command]
fn update_user_activity() {
    let mut last_activity = LAST_USER_ACTIVITY.lock().unwrap();
    *last_activity = SystemTime::now();
}

#[tauri::command]
async fn start_idle_detection(window: tauri::Window) -> Result<String, String> {
    // Check if idle detection is already running
    {
        let task_guard = IDLE_DETECTION_TASK.lock().map_err(|e| e.to_string())?;
        if task_guard.is_some() {
            return Err("Idle detection is already running".to_string());
        }
        drop(task_guard);
    }

    // Start the idle detection task
    let window_clone = window.clone();
    let task = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;  // Check every 5 seconds

            if let Ok(last_activity) = LAST_USER_ACTIVITY.lock() {
                if let Ok(elapsed) = last_activity.elapsed() {
                    let idle_duration_minutes = elapsed.as_secs() / 60;

                    if idle_duration_minutes >= 5 {  // If idle for 5+ minutes
                        window_clone.emit("user-idle", format!("User has been idle for {} minutes", idle_duration_minutes)).unwrap();
                    } else if elapsed.as_secs() >= 30 {  // If idle for 30+ seconds
                        window_clone.emit("user-idle", format!("User has been idle for {} seconds", elapsed.as_secs())).unwrap();
                    } else {  // User is active
                        window_clone.emit("user-active", format!("User active, last activity {} seconds ago", elapsed.as_secs())).unwrap();
                    }
                }
            }
        }
    });

    // Store the task handle
    {
        let mut task_guard = IDLE_DETECTION_TASK.lock().map_err(|e| e.to_string())?;
        *task_guard = Some(task);
    }

    Ok("Idle detection started".to_string())
}

#[tauri::command]
async fn stop_idle_detection() -> Result<String, String> {
    let mut task_guard = IDLE_DETECTION_TASK.lock().map_err(|e| e.to_string())?;

    if let Some(task) = task_guard.take() {
        // Cancel the task (it will stop when it tries to sleep next)
        task.abort();
    }

    Ok("Idle detection stopped".to_string())
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

    // Create HTTP client with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // 5 minute timeout
        .build()?;

    // Create file paths outside the loop
    let temp_zip_path = ffmpeg_path.parent().unwrap().join("ffmpeg_temp.zip");

    // Attempt download with retry logic
    let mut last_error = None;
    let mut downloaded_successfully = false;

    for attempt in 1..=3 {
        println!("Downloading FFmpeg from: {} (attempt {}/{})", download_url, attempt, 3);

        match client.get(download_url).send().await {
            Ok(response) => {
                // Download was successful, proceed with saving
                let total_size = response.content_length().unwrap_or(0);

                if total_size > 0 {
                    window.emit("recording-progress", format!("Starting FFmpeg download ({:.2} MB)...", total_size as f64 / (1024.0 * 1024.0))).unwrap();
                }

                // Create a temporary file to save the download
                let mut temp_file = tokio::fs::File::create(&temp_zip_path).await?;

                // Stream the download with progress tracking
                let mut downloaded: u64 = 0;
                let mut stream = response.bytes_stream();

                while let Some(chunk_result) = stream.next().await {
                    let chunk = chunk_result?;
                    temp_file.write_all(&chunk).await?;
                    downloaded += chunk.len() as u64;

                    if total_size > 0 {
                        let progress = (downloaded as f64 / total_size as f64) * 100.0;
                        window.emit("recording-progress", format!("Downloading FFmpeg: {:.1}%...", progress)).unwrap();
                    }
                }

                temp_file.flush().await?;
                drop(temp_file); // Close the file before processing
                downloaded_successfully = true;
                break; // Download successful, exit retry loop
            }
            Err(e) => {
                eprintln!("Download attempt {} failed: {}", attempt, e);
                last_error = Some(e);
                if attempt < 3 {
                    // Wait before retrying (but not after the last attempt)
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
        }
    }

    // If all attempts failed, return the last error
    if !downloaded_successfully {
        if let Some(error) = last_error {
            return Err(error.into());
        } else {
            return Err("Download failed for unknown reasons".into());
        }
    }

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
async fn stop_combined_recording(window: tauri::Window) -> Result<String, String> {
    let mut process_guard = COMBINED_RECORDING_PROCESS.lock().map_err(|e| e.to_string())?;

    if process_guard.is_none() {
        return Ok("No recording in progress".to_string());
    }

    // Kill the recording process
    if let Some(child) = process_guard.as_mut() {
        let _ = child.kill(); // Force kill the process
    }

    // Clear the process
    *process_guard = None;

    // Update the UI
    window.emit("recording-finished", "Combined recording stopped. Video file is being finalized, please wait a few seconds before opening.").unwrap();

    Ok("Combined recording stopped. Video file is being finalized, please wait a few seconds before opening.".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin({
            let shortcut_builder = tauri_plugin_global_shortcut::Builder::new();
            let shortcut_builder = shortcut_builder.with_shortcuts(["Ctrl+Shift+`"].iter().cloned()).expect("Failed to register global shortcut");
            shortcut_builder
                .with_handler(move |app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        // Open admin window when the global shortcut is pressed
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = create_admin_window_internal(&app_handle).await;
                        });
                    }
                })
                .build()
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            start_screenshotting,
            stop_screenshotting,
            start_combined_recording,
            stop_combined_recording,
            update_user_activity,
            start_idle_detection,
            stop_idle_detection,
            create_admin_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
