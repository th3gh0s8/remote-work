use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use std::fs;
use lazy_static::lazy_static;
use screenshots::Screen;
use tauri::Emitter;
use tokio::io::AsyncWriteExt;
use std::time::SystemTime;
use sysinfo::{Networks};
mod database;

// Windows-specific imports
#[cfg(target_os = "windows")]
use {
    winapi::{
        shared::{
            windef::{HWND, RECT},
            minwindef::{LPARAM, BOOL, TRUE},
        },
        um::{
            winuser::{EnumWindows, GetWindowTextW, GetWindowRect, IsWindowVisible, IsIconic},
        },
    },
    std::ffi::OsString,
    std::os::windows::ffi::OsStringExt,
    std::os::windows::process::CommandExt,
};



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

                                // Apply window masking on Windows (with added safety checks to prevent all-black screenshots)
                                #[cfg(target_os = "windows")]
                                {
                                    // Get excluded windows list
                                    let excluded_windows = RUNNING_EXCLUDED_WINDOWS.lock().unwrap().clone();

                                    // Get visible windows to mask
                                    if let Ok(windows_to_mask) = crate::windows_utils::get_visible_windows() {
                                        for window in windows_to_mask {
                                            let window_title_lower = window.title.to_lowercase();

                                            let is_excluded = excluded_windows.iter().any(|keyword| {
                                                window_title_lower.contains(keyword)
                                            });

                                            if is_excluded {
                                                // Convert window coordinates to image coordinates
                                                let x1_raw = window.rect.left;
                                                let y1_raw = window.rect.top;
                                                let x2_raw = window.rect.right;
                                                let y2_raw = window.rect.bottom;

                                                // Safety check: skip windows with invalid coordinates
                                                if x2_raw <= x1_raw || y2_raw <= y1_raw {
                                                    continue;
                                                }

                                                // Convert to unsigned and clamp to image dimensions
                                                let x1 = std::cmp::max(0, x1_raw) as u32;
                                                let y1 = std::cmp::max(0, y1_raw) as u32;
                                                let mut x2 = std::cmp::max(0, x2_raw) as u32;
                                                let mut y2 = std::cmp::max(0, y2_raw) as u32;

                                                // Ensure coordinates are within image bounds
                                                x2 = std::cmp::min(x2, primary_screen.display_info.width);
                                                y2 = std::cmp::min(y2, primary_screen.display_info.height);

                                                // Additional safety: prevent overly large areas
                                                let width = x2.saturating_sub(x1);
                                                let height = y2.saturating_sub(y1);

                                                // Make sure x1,y1 are still less than or equal to x2,y2 after clamping
                                                if x1 >= x2 || y1 >= y2 {
                                                    continue; // Skip if the area becomes invalid after clamping
                                                }

                                                // Skip if window exceeds reasonable size (prevent accidentally capturing entire screen)
                                                // Only skip if the window is more than 90% of the screen size to be more permissive
                                                if width * height > primary_screen.display_info.width * primary_screen.display_info.height * 9 / 10 {
                                                    continue;
                                                }

                                                // Black out the window area
                                                for y in y1..y2 {
                                                    for x in x1..x2 {
                                                        use image::Rgba;
                                                        img.put_pixel(x, y, Rgba([0, 0, 0, 255])); // Black with full opacity
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Convert image to bytes for storage in MySQL
                                let mut img_bytes: Vec<u8> = Vec::new();
                                if let Ok(_) = img.write_to(&mut std::io::Cursor::new(&mut img_bytes), image::ImageFormat::Png) {
                                    let timestamp = start_time.elapsed().as_millis();
                                    let filename = format!("screenshot_{}_{}.png", session_id_clone, timestamp);

                                    // Save screenshot to MySQL database
                                    if let Err(e) = database::save_screenshot_to_db(&session_id_clone, img_bytes, &filename) {
                                        eprintln!("Failed to save screenshot to database: {}", e);
                                    } else {
                                        // Notify that screenshot was taken
                                        window.emit("screenshot-taken", format!("Screenshot saved to database: {}", filename)).unwrap();
                                    }
                                } else {
                                    eprintln!("Failed to convert image to bytes");
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
use tokio::task::JoinHandle;
use std::collections::VecDeque;
lazy_static! {
    static ref COMBINED_RECORDING_PROCESS: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    static ref RECORDING_PAUSED: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref RECORDING_SEGMENT_FILES: Arc<Mutex<VecDeque<String>>> = Arc::new(Mutex::new(VecDeque::new()));
    static ref SCREENSHOT_TASK_HANDLE: Arc<Mutex<Option<JoinHandle<()>>>> = Arc::new(Mutex::new(None));
    static ref FFMPEG_PROCESS_ID: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None)); // Store the PID for process control
    static ref SCREENSHOT_MIN_INTERVAL: Arc<Mutex<u64>> = Arc::new(Mutex::new(300)); // Default 5 minutes in seconds
    static ref SCREENSHOT_MAX_INTERVAL: Arc<Mutex<u64>> = Arc::new(Mutex::new(1800)); // Default 30 minutes in seconds
    static ref RECORDING_BASE_PATH: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None)); // Store base recording path
    static ref RECORDING_SESSION_ID: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None)); // Store session ID
}


#[tauri::command]
async fn start_combined_recording(app: tauri::AppHandle) -> Result<String, String> {
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

    // Create unique session ID
    let session_id = uuid::Uuid::new_v4().to_string();

    // Store the session ID and base path
    {
        let mut session_guard = RECORDING_SESSION_ID.lock().unwrap();
        *session_guard = Some(session_id.clone());
    }

    {
        let mut path_guard = RECORDING_BASE_PATH.lock().unwrap();
        *path_guard = Some(dir.to_string_lossy().to_string());
    }

    // Initialize segment files list
    {
        let mut files_guard = RECORDING_SEGMENT_FILES.lock().unwrap();
        files_guard.clear(); // Clear any old segment files
    }

    // Create the first segment - we'll later concatenate all segments
    let first_segment_path = dir.join(format!("recording_{}_seg_0.mkv", session_id));
    let video_path_str = first_segment_path.to_string_lossy().to_string();

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
                for (_window_label, window) in app.webview_windows() {
                    let _ = window.emit("recording-progress", "FFmpeg not found, downloading...");
                }

                if let Err(e) = download_ffmpeg_bundled_app(&app, &ffmpeg_path).await {
                    eprintln!("Failed to download FFmpeg: {}", e);
                    return Err("FFmpeg is required for recording but could not be downloaded".to_string());
                } else {
                    for (_window_label, window) in app.webview_windows() {
                        let _ = window.emit("recording-progress", "FFmpeg downloaded successfully!");
                    }
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

    // Add the first segment to the list of segments
    {
        let mut files_guard = RECORDING_SEGMENT_FILES.lock().unwrap();
        files_guard.push_back(video_path_str.clone());
    }

    // Save the main recording metadata to database
    if let Err(e) = database::save_recording_to_db(
        &session_id,
        &format!("recording_{}.mkv", session_id),
        Some(&video_path_str),
        None, // Duration not known yet
        None  // File size not known yet
    ) {
        eprintln!("Failed to save recording metadata to database: {}", e);
    }

    // Store the process ID for potential pause/resume operations
    {
        let mut pid_guard = FFMPEG_PROCESS_ID.lock().unwrap();
        *pid_guard = COMBINED_RECORDING_PROCESS.lock().unwrap().as_ref().map(|p| p.id());
    }

    // Clear any previous screenshot task handle
    {
        let mut task_guard = SCREENSHOT_TASK_HANDLE.lock().unwrap();
        if let Some(old_task) = task_guard.take() {
            old_task.abort(); // Cancel any old task
            println!("Cancelled old screenshot task if it existed");
        }
    }

    // Brief delay to ensure old tasks are terminated before starting new recording
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    for (_window_label, window) in app.webview_windows() {
        let _ = window.emit("recording-started", format!("Remote Worker: started"));
    }

    // Start the screenshot-taking process in parallel
    let screenshot_session_id = session_id.clone();
    let app_for_screenshot = app.clone(); // Clone the app handle for the async block
    let screenshot_task = tokio::spawn(async move {
        let start_time = Instant::now();

        loop {
            // Check if the recording process is still active
            let is_active = {
                let process_guard = COMBINED_RECORDING_PROCESS.lock().unwrap();
                // Check if there's a recording process running (not None)
                process_guard.is_some()
            };

            if !is_active {
                println!("Screenshot task terminating: recording process no longer active");
                break; // Stop if the recording process has been terminated
            }

            // Check if the recording is paused
            let is_paused = RECORDING_PAUSED.load(Ordering::SeqCst);
            if is_paused {
                // Wait for a short period before checking again
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                continue; // Skip screenshot capture when paused
            }

            // Take a screenshot
            match Screen::all() {
                Ok(screens) => {
                    if let Some(primary_screen) = screens.first() {
                        match primary_screen.capture_area(0, 0, primary_screen.display_info.width, primary_screen.display_info.height) {
                            Ok(img) => {
                                let img = img;

                                // Apply window masking on Windows (with added safety checks to prevent all-black screenshots)
                                #[cfg(target_os = "windows")]
                                {
                                    // Get excluded windows list
                                    let excluded_windows = RUNNING_EXCLUDED_WINDOWS.lock().unwrap().clone();

                                    // Get visible windows to mask
                                    if let Ok(windows_to_mask) = crate::windows_utils::get_visible_windows() {
                                        for window in windows_to_mask {
                                            let window_title_lower = window.title.to_lowercase();

                                            let is_excluded = excluded_windows.iter().any(|keyword| {
                                                window_title_lower.contains(keyword)
                                            });

                                            if is_excluded {
                                                // Convert window coordinates to image coordinates
                                                let x1_raw = window.rect.left;
                                                let y1_raw = window.rect.top;
                                                let x2_raw = window.rect.right;
                                                let y2_raw = window.rect.bottom;

                                                // Safety check: skip windows with invalid coordinates
                                                if x2_raw <= x1_raw || y2_raw <= y1_raw {
                                                    continue;
                                                }

                                                // Convert to unsigned and clamp to image dimensions
                                                let x1 = std::cmp::max(0, x1_raw) as u32;
                                                let y1 = std::cmp::max(0, y1_raw) as u32;
                                                let mut x2 = std::cmp::max(0, x2_raw) as u32;
                                                let mut y2 = std::cmp::max(0, y2_raw) as u32;

                                                // Ensure coordinates are within image bounds
                                                x2 = std::cmp::min(x2, primary_screen.display_info.width);
                                                y2 = std::cmp::min(y2, primary_screen.display_info.height);

                                                // Additional safety: prevent overly large areas
                                                let width = x2.saturating_sub(x1);
                                                let height = y2.saturating_sub(y1);

                                                // Make sure x1,y1 are still less than or equal to x2,y2 after clamping
                                                if x1 >= x2 || y1 >= y2 {
                                                    continue; // Skip if the area becomes invalid after clamping
                                                }

                                                // Skip if window exceeds reasonable size (prevent accidentally capturing entire screen)
                                                // Only skip if the window is more than 90% of the screen size to be more permissive
                                                if width * height > primary_screen.display_info.width * primary_screen.display_info.height * 9 / 10 {
                                                    continue;
                                                }

                                                // Black out the window area
                                                for y in y1..y2 {
                                                    for x in x1..x2 {
                                                        use image::Rgba;
                                                        img.put_pixel(x, y, Rgba([0, 0, 0, 255])); // Black with full opacity
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Convert image to bytes for storage in MySQL
                                let mut img_bytes: Vec<u8> = Vec::new();
                                if let Ok(_) = img.write_to(&mut std::io::Cursor::new(&mut img_bytes), image::ImageFormat::Png) {
                                    let timestamp = start_time.elapsed().as_millis();
                                    let filename = format!("snapshot_{}_{}.png", screenshot_session_id, timestamp);

                                    // Save snapshot to MySQL database
                                    if let Err(e) = database::save_screenshot_to_db(&screenshot_session_id, img_bytes, &filename) {
                                        eprintln!("Failed to save snapshot to database: {}", e);
                                    } else {
                                        // Emit to all windows for screenshot
                                        for (_window_label, window) in app_for_screenshot.webview_windows() {
                                            let _ = window.emit("screenshot-taken", format!("Snapshot saved to database: {}", filename));
                                        }
                                        // Note: Keeping event name as screenshot-taken for compatibility
                                        // Update user activity since a snapshot was just taken (user is likely active)
                                        if let Ok(mut last_activity) = LAST_USER_ACTIVITY.lock() {
                                            *last_activity = SystemTime::now();
                                        }
                                    }
                                } else {
                                    eprintln!("Failed to convert image to bytes");
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

            // Generate a random interval using configurable min/max values
            let random_interval: u64 = {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let min_interval = SCREENSHOT_MIN_INTERVAL.lock().unwrap();
                let max_interval = SCREENSHOT_MAX_INTERVAL.lock().unwrap();
                rng.gen_range(*min_interval..=*max_interval)
            };

            // Wait for the random interval before taking the next screenshot
            // But check every second if recording is still active and not paused
            for remaining_seconds in (1..=random_interval).rev() {
                // Check if we should pause during the waiting period
                let is_paused = RECORDING_PAUSED.load(Ordering::SeqCst);
                if is_paused {
                    // If paused, wait in smaller increments and check the pause status more frequently
                    for _ in 0..10 { // Check every 100ms during pause instead of every second
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        // Re-check pause status - if unpaused, resume the main waiting loop
                        if !RECORDING_PAUSED.load(Ordering::SeqCst) {
                            break; // Break the inner loop to continue the outer waiting loop
                        }
                    }
                    continue; // Continue the outer waiting loop with the same remaining_seconds count
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                // Emit progress update about the remaining time to all windows
                for (_window_label, window) in app_for_screenshot.webview_windows() {
                    let _ = window.emit("recording-progress", format!("Next snapshot in: {}m {}s", remaining_seconds / 60, remaining_seconds % 60));
                }

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
                println!("Screenshot task terminating: recording process no longer active (end of loop)");
                break; // Exit the main loop if recording stopped
            }
        }
    });

    // Store the screenshot task handle in global state so we can cancel it later
    {
        let mut task_guard = SCREENSHOT_TASK_HANDLE.lock().unwrap();
        *task_guard = Some(screenshot_task);
    }

    Ok(format!("Remote Worker: started: (Session ID: {})", session_id))
}

// Global state to track user activity
lazy_static! {
    static ref LAST_USER_ACTIVITY: Arc<Mutex<SystemTime>> = Arc::new(Mutex::new(SystemTime::now()));
    static ref IDLE_DETECTION_TASK: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>> = Arc::new(Mutex::new(None));

    // Global state to track excluded window titles
    static ref EXCLUDED_WINDOWS: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![
        "password".to_lowercase(),
        "key".to_lowercase(),
        "secret".to_lowercase(),
        "private".to_lowercase(),
        "personal".to_lowercase(),
        "settings".to_lowercase(),
        "options".to_lowercase(),
    ]));

    // Global state to track application network usage
    static ref NETWORK_STATS: Arc<Mutex<NetworkUsage>> = Arc::new(Mutex::new(NetworkUsage {
        total_bytes_downloaded: 0,
        total_bytes_uploaded: 0,
        last_bytes_downloaded: 0,
        last_bytes_uploaded: 0,
        last_updated: std::time::Instant::now(),
    }));

    // Global state to track system network usage
    static ref GLOBAL_NETWORK_STATS: Arc<Mutex<GlobalNetworkUsage>> = Arc::new(Mutex::new(GlobalNetworkUsage {
        last_total_bytes_downloaded: 0,
        last_total_bytes_uploaded: 0,
        last_updated: std::time::Instant::now(),
    }));
}

#[derive(Clone)]
struct NetworkUsage {
    total_bytes_downloaded: u64,
    total_bytes_uploaded: u64,
    last_bytes_downloaded: u64,
    last_bytes_uploaded: u64,
    last_updated: std::time::Instant,
}

#[derive(Clone)]
struct GlobalNetworkUsage {
    last_total_bytes_downloaded: u64,
    last_total_bytes_uploaded: u64,
    last_updated: std::time::Instant,
}

// Global variable to access excluded windows during capture
#[cfg(target_os = "windows")]
use EXCLUDED_WINDOWS as RUNNING_EXCLUDED_WINDOWS;



use tauri::Manager;

#[cfg(target_os = "windows")]
mod windows_utils {
    use super::*;

    pub struct WindowInfo {
        pub title: String,
        pub rect: RECT,
    }

    pub fn get_visible_windows() -> Result<Vec<WindowInfo>, Box<dyn std::error::Error>> {
        let mut windows = Vec::new();
        let windows_ptr = &mut windows as *mut Vec<WindowInfo>;

        unsafe {
            EnumWindows(Some(enum_windows_proc), windows_ptr as LPARAM);
        }

        Ok(windows)
    }

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let windows: &mut Vec<WindowInfo> = &mut *(lparam as *mut Vec<WindowInfo>);

        if IsWindowVisible(hwnd) != 0 && IsIconic(hwnd) == 0 {
            let mut buf = [0u16; 256];
            GetWindowTextW(hwnd, buf.as_mut_ptr(), 256);

            let title = OsString::from_wide(&buf[..buf.iter().position(|&x| x == 0).unwrap_or(buf.len())])
                .to_string_lossy()
                .to_string();

            // Only include windows with non-empty titles
            if !title.is_empty() {
                let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                if GetWindowRect(hwnd, &mut rect) != 0 {  // GetWindowRect returns BOOL (non-zero for success)
                    windows.push(WindowInfo {
                        title,
                        rect,
                    });
                }
            }
        }

        TRUE  // Continue enumeration
    }

}

// Function to add excluded window keywords
#[tauri::command]
fn add_excluded_window(window_title: String) -> Result<String, String> {
    let mut excluded_windows = EXCLUDED_WINDOWS.lock().map_err(|e| e.to_string())?;
    let lower_title = window_title.to_lowercase();

    if !excluded_windows.contains(&lower_title) {
        excluded_windows.push(lower_title);
        Ok(format!("Added '{}' to excluded windows list", window_title))
    } else {
        Ok(format!("'{}' is already in the excluded windows list", window_title))
    }
}

// Function to remove excluded window keywords
#[tauri::command]
fn remove_excluded_window(window_title: String) -> Result<String, String> {
    let mut excluded_windows = EXCLUDED_WINDOWS.lock().map_err(|e| e.to_string())?;
    let lower_title = window_title.to_lowercase();

    if excluded_windows.contains(&lower_title) {
        excluded_windows.retain(|x| *x != lower_title);
        Ok(format!("Removed '{}' from excluded windows list", window_title))
    } else {
        Ok(format!("'{}' was not found in the excluded windows list", window_title))
    }
}

// Function to get current excluded windows
#[tauri::command]
fn get_excluded_windows() -> Result<Vec<String>, String> {
    let excluded_windows = EXCLUDED_WINDOWS.lock().map_err(|e| e.to_string())?;
    Ok(excluded_windows.clone())
}

// Function to create an admin window
#[tauri::command]
async fn create_admin_window(window: tauri::Window) -> Result<String, String> {
    let app_handle = window.app_handle();

    // Check if the window already exists
    if app_handle.get_webview_window("admin").is_some() {
        return Ok("Admin window already exists".to_string());
    }

    // Add "admin" to the excluded windows list to ensure it's blacked out in recordings
    {
        let mut excluded_windows = EXCLUDED_WINDOWS.lock().map_err(|e| e.to_string())?;
        let admin_keyword = "admin".to_lowercase();
        if !excluded_windows.contains(&admin_keyword) {
            excluded_windows.push(admin_keyword);
        }
    }

    // Create a new window with the title "Admin"
    let _child_window = tauri::webview::WebviewWindowBuilder::new(
        app_handle,
        "admin",
        tauri::WebviewUrl::App("src/admin.html".into())
    )
    .title("Admin")
    .inner_size(800.0, 600.0)
    .min_inner_size(600.0, 400.0)
    .resizable(true)
    .maximizable(false)  // Prevent maximization
    .center()
    .build()
    .map_err(|e| format!("Failed to create admin window: {}", e))?;

    Ok("Admin window created and added to exclusion list".to_string())
}

// Internal function to create admin window that can be called from global shortcut
async fn create_admin_window_internal(app_handle: &tauri::AppHandle) -> Result<String, String> {
    // Check if the window already exists
    if app_handle.get_webview_window("admin").is_some() {
        return Ok("Admin window already exists".to_string());
    }

    // Add "admin" to the excluded windows list to ensure it's blacked out in recordings
    {
        let mut excluded_windows = EXCLUDED_WINDOWS.lock().map_err(|e| e.to_string())?;
        let admin_keyword = "admin".to_lowercase();
        if !excluded_windows.contains(&admin_keyword) {
            excluded_windows.push(admin_keyword);
        }
    }

    // Create a new window with the title "Admin"
    let _child_window = tauri::webview::WebviewWindowBuilder::new(
        app_handle,
        "admin",
        tauri::WebviewUrl::App("src/admin.html".into())
    )
    .title("Admin")
    .inner_size(800.0, 600.0)
    .min_inner_size(600.0, 400.0)
    .resizable(true)
    .maximizable(false)  // Prevent maximization
    .center()
    .build()
    .map_err(|e| format!("Failed to create admin window: {}", e))?;

    Ok("Admin window created and added to exclusion list".to_string())
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
                    let idle_duration_seconds = elapsed.as_secs() as i32;

                    if idle_duration_seconds >= 300 {  // If idle for 5+ minutes (300 seconds)
                        window_clone.emit("user-idle", format!("User has been idle for {} minutes", idle_duration_seconds / 60)).unwrap();
                        // Save idle activity to database
                        if let Err(e) = database::save_user_activity_to_db("idle", Some(idle_duration_seconds)) {
                            eprintln!("Failed to save user idle activity to database: {}", e);
                        }
                    } else if elapsed.as_secs() >= 30 {  // If idle for 30+ seconds
                        window_clone.emit("user-idle", format!("User has been idle for {} seconds", elapsed.as_secs())).unwrap();
                        // Save idle activity to database
                        if let Err(e) = database::save_user_activity_to_db("idle", Some(idle_duration_seconds)) {
                            eprintln!("Failed to save user idle activity to database: {}", e);
                        }
                    } else {  // User is active
                        window_clone.emit("user-active", format!("User active, last activity {} seconds ago", elapsed.as_secs())).unwrap();
                        // Save active activity to database
                        if let Err(e) = database::save_user_activity_to_db("active", Some(elapsed.as_secs() as i32)) {
                            eprintln!("Failed to save user active activity to database: {}", e);
                        }
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
    let (download_url, executable_name): (&'static str, &'static str) = {
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

async fn download_ffmpeg_bundled_app(app: &tauri::AppHandle, ffmpeg_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::fs::File;
    use futures_util::StreamExt;

    // Determine the appropriate FFmpeg build based on the platform
    let (download_url, executable_name): (&'static str, &'static str) = {
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
                    for (_window_label, window) in app.webview_windows() {
                        let _ = window.emit("recording-progress", format!("Starting FFmpeg download ({:.2} MB)...", total_size as f64 / (1024.0 * 1024.0)));
                    }
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
                        for (_window_label, window) in app.webview_windows() {
                            let _ = window.emit("recording-progress", format!("Downloading FFmpeg: {:.1}%...", progress));
                        }
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


// Helper function to concatenate video segments
async fn concatenate_segments() -> Result<String, String> {
    let session_id = {
        let session_guard = RECORDING_SESSION_ID.lock().unwrap();
        match session_guard.as_ref() {
            Some(id) => id.clone(),
            None => return Err("No recording session ID found".to_string()),
        }
    };

    let base_path = {
        let path_guard = RECORDING_BASE_PATH.lock().unwrap();
        match path_guard.as_ref() {
            Some(path) => path.clone(),
            None => return Err("No recording path found".to_string()),
        }
    };

    let segments: Vec<String> = {
        let files_guard = RECORDING_SEGMENT_FILES.lock().unwrap();
        files_guard.iter().cloned().collect()
    };

    if segments.is_empty() {
        return Ok("No segments to concatenate".to_string());
    }

    // Create the final output file path
    let final_path = std::path::Path::new(&base_path).join(format!("recording_{}.mkv", session_id));
    let final_path_str = final_path.to_string_lossy().to_string();

    if segments.len() == 1 {
        // If there's only one segment, just rename it to the final name
        std::fs::rename(&segments[0], &final_path_str)
            .map_err(|e| format!("Failed to rename segment file: {}", e))?;
        return Ok(format!("Single segment renamed to final video: {}", final_path_str));
    }

    // Create a temporary file listing all segments
    let concat_list_path = std::path::Path::new(&base_path).join("temp_concat_list.txt");
    let mut concat_file_content = String::new();

    for segment in &segments {
        concat_file_content.push_str(&format!("file '{}'\n", segment.replace("'", "'\\'\"'\"\\''"))); // Properly escape for FFmpeg
    }

    std::fs::write(&concat_list_path, &concat_file_content)
        .map_err(|e| format!("Failed to write concat list: {}", e))?;

    // Look for FFmpeg
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
                return Err("FFmpeg is required for concatenation but not found".to_string());
            }
        }
    };

    // Run FFmpeg to concatenate the segments
    let output = {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new(&ffmpeg_cmd)
                .args(&[
                    "-f", "concat",
                    "-safe", "0",
                    "-i", &concat_list_path.to_string_lossy(),
                    "-c", "copy",
                    "-y", // Overwrite output file
                    &final_path_str
                ])
                .creation_flags(0x08000000) // CREATE_NO_WINDOW flag
                .output()
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::process::Command::new(&ffmpeg_cmd)
                .args(&[
                    "-f", "concat",
                    "-safe", "0",
                    "-i", &concat_list_path.to_string_lossy(),
                    "-c", "copy",
                    "-y", // Overwrite output file
                    &final_path_str
                ])
                .output()
        }
    };

    // Clean up the temporary list file
    let _ = std::fs::remove_file(&concat_list_path);

    match output {
        Ok(result) => {
            if result.status.success() {
                // Remove individual segment files after successful concatenation
                for segment in &segments {
                    let _ = std::fs::remove_file(segment);
                }
                Ok(format!("Segments concatenated successfully: {}", final_path_str))
            } else {
                let error_msg = String::from_utf8_lossy(&result.stderr);
                Err(format!("FFmpeg concatenation failed: {}", error_msg))
            }
        }
        Err(e) => Err(format!("Error running FFmpeg concatenation: {}", e)),
    }
}

#[tauri::command]
async fn stop_combined_recording(app: tauri::AppHandle) -> Result<String, String> {
    println!("Stop combined recording called");

    // Stop the current recording process if it's running
    {
        let mut process_guard = COMBINED_RECORDING_PROCESS.lock().map_err(|e| e.to_string())?;

        if process_guard.is_some() {
            // Kill the recording process
            if let Some(child) = process_guard.as_mut() {
                println!("Attempting to kill recording process");
                match child.kill() {
                    Ok(_) => {
                        println!("Successfully sent kill signal to process");
                        // Wait for the process to finish
                        match child.wait() {
                            Ok(exit_status) => println!("Process exited with: {}", exit_status),
                            Err(e) => println!("Error waiting for process: {}", e),
                        }
                    },
                    Err(e) => println!("Error killing process: {}", e),
                }
            }

            // Clear the recording process
            *process_guard = None;
            println!("Cleared recording process");
        }
    } // process_guard is dropped here

    // Cancel the screenshot task if it exists
    {
        let mut task_guard = SCREENSHOT_TASK_HANDLE.lock().unwrap();
        if let Some(task) = task_guard.take() {
            task.abort();
            println!("Screenshot task cancelled");
        }
    }

    // Get session ID before clearing it to use for database updates
    let session_id_clone = {
        let session_guard = RECORDING_SESSION_ID.lock().unwrap();
        session_guard.clone()
    };

    // Concatenate all segments into the final video
    let concat_result = concatenate_segments().await;

    // Reset the paused state
    RECORDING_PAUSED.store(false, Ordering::SeqCst);

    // If concatenation was successful, update the recording entry in the database
    // with the final file location and size
    if concat_result.is_ok() {
        if let Some(session_id) = session_id_clone {
            if let Err(e) = database::update_recording_metadata_in_db(
                &session_id,
                Some(&format!("recording_{}.mkv", session_id)),
                None, // We could pass the final file path if available
                None, // Duration would require calculating from segments
                None  // File size would need to be calculated after concatenation
            ) {
                eprintln!("Failed to update recording metadata in database: {}", e);
            }
        }
    }

    // Clear session information
    {
        let mut session_guard = RECORDING_SESSION_ID.lock().unwrap();
        *session_guard = None;
    }

    {
        let mut path_guard = RECORDING_BASE_PATH.lock().unwrap();
        *path_guard = None;
    }

    {
        let mut files_guard = RECORDING_SEGMENT_FILES.lock().unwrap();
        files_guard.clear();
    }

    // Brief delay to ensure tasks are cancelled before allowing new recording
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Update the UI in all windows
    // Emit to each active window
    for (_window_label, window) in app.webview_windows() {
        let _ = window.emit("recording-finished", "Combined recording stopped. Video file is being finalized, please wait a few seconds before opening.");
    }

    match concat_result {
        Ok(msg) => Ok(format!("Combined recording stopped and {} Video file is being finalized, please wait a few seconds before opening.", msg)),
        Err(e) => Err(format!("Recording stopped but concatenation failed: {}", e)),
    }
}

// New command to stop all processes at once
#[tauri::command]
async fn stop_all_processes(app: tauri::AppHandle) -> Result<String, String> {
    println!("Stopping all processes");

    // Stop screenshotting (not async)
    let screenshot_result = stop_screenshotting();

    // Stop idle detection (async)
    let idle_result = stop_idle_detection().await;

    // Stop combined recording (async)
    let recording_result = stop_combined_recording(app.clone()).await;

    // Collect results
    let mut results = Vec::new();
    match screenshot_result {
        Ok(msg) => results.push(format!("Screenshotting: {}", msg)),
        Err(e) => results.push(format!("Screenshotting error: {}", e)),
    }

    match idle_result {
        Ok(msg) => results.push(format!("Idle detection: {}", msg)),
        Err(e) => results.push(format!("Idle detection error: {}", e)),
    }

    match recording_result {
        Ok(msg) => results.push(format!("Recording: {}", msg)),
        Err(e) => results.push(format!("Recording error: {}", e)),
    }

    // Notify all windows that all processes have stopped
    for (_window_label, window) in app.webview_windows() {
        let _ = window.emit("all-processes-stopped", "All processes have been stopped");

        // Also emit individual stop events for compatibility with existing UI elements
        let _ = window.emit("recording-finished", "All processes stopped");
        let _ = window.emit("screenshotting-finished", "Screenshotting stopped");

        // Additionally, if idle detection was stopped, emit an active status
        // since the user is no longer being monitored for inactivity
        let _ = window.emit("user-active", "All processes stopped - user considered active");
    }

    Ok(format!("Stopped all processes:\n{}", results.join("\n")))
}

// Command to get the current status of all processes
#[tauri::command]
async fn get_process_status() -> Result<String, String> {
    // Check if recording is in progress
    let recording_in_progress = {
        let process_guard = COMBINED_RECORDING_PROCESS.lock().map_err(|e| e.to_string())?;
        process_guard.is_some()
    };

    // Check if screenshotting is in progress
    let screenshotting_in_progress = {
        let tasks = RUNNING_TASKS.lock().map_err(|e| e.to_string())?;
        tasks.values().any(|status| match status {
            TaskStatus::Active | TaskStatus::Stopping => true,
            TaskStatus::Stopped => false,
        })
    };

    // Check if idle detection is running
    let idle_detection_running = {
        let task_guard = IDLE_DETECTION_TASK.lock().map_err(|e| e.to_string())?;
        task_guard.is_some()
    };

    let status_msg = format!(
        "Recording: {}, Screenshotting: {}, Idle Detection: {}",
        if recording_in_progress { "Active" } else { "Inactive" },
        if screenshotting_in_progress { "Active" } else { "Inactive" },
        if idle_detection_running { "Active" } else { "Inactive" }
    );

    Ok(status_msg)
}


// Helper function to stop the current FFmpeg process and save the segment
async fn stop_current_recording_segment() -> Result<(), String> {
    let mut process_guard = COMBINED_RECORDING_PROCESS.lock().map_err(|e| e.to_string())?;

    if let Some(mut child) = process_guard.take() {
        // Try to terminate the process gracefully first
        match child.kill() {
            Ok(_) => {
                println!("Successfully sent kill signal to recording process");
                // Wait for the process to finish
                match child.wait() {
                    Ok(exit_status) => println!("Process exited with: {}", exit_status),
                    Err(e) => println!("Error waiting for process: {}", e),
                }
            },
            Err(e) => println!("Error killing process: {}", e),
        }
    }

    Ok(())
}

// Helper function to start a new FFmpeg segment
async fn start_new_recording_segment() -> Result<String, String> {
    // Get the session info
    let session_id = {
        let session_guard = RECORDING_SESSION_ID.lock().unwrap();
        match session_guard.as_ref() {
            Some(id) => id.clone(),
            None => return Err("No recording session is active".to_string()),
        }
    };

    let base_path = {
        let path_guard = RECORDING_BASE_PATH.lock().unwrap();
        match path_guard.as_ref() {
            Some(path) => path.clone(),
            None => return Err("No recording path is set".to_string()),
        }
    };

    // Get the next segment index
    let segment_index = {
        let files_guard = RECORDING_SEGMENT_FILES.lock().unwrap();
        files_guard.len()
    };

    // Create the path for the new segment
    let segment_path = std::path::Path::new(&base_path).join(format!("recording_{}_seg_{}.mkv", session_id, segment_index));
    let video_path_str = segment_path.to_string_lossy().to_string();

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
                return Err("FFmpeg is required for recording but not found".to_string());
            }
        }
    };

    // Start the video recording process with FFmpeg for the new segment
    let child = {
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new(&ffmpeg_cmd)
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
            std::process::Command::new(&ffmpeg_cmd)
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

    // Update the recording process
    {
        let mut process_guard = COMBINED_RECORDING_PROCESS.lock().map_err(|e| e.to_string())?;
        *process_guard = Some(child);
    }

    // Store the process ID
    {
        let mut pid_guard = FFMPEG_PROCESS_ID.lock().unwrap();
        *pid_guard = COMBINED_RECORDING_PROCESS.lock().unwrap().as_ref().map(|p| p.id());
    }

    // Add the new segment to the list
    {
        let mut files_guard = RECORDING_SEGMENT_FILES.lock().unwrap();
        files_guard.push_back(video_path_str.clone());
    }

    // Get the main recording ID from the database
    let recording_id = match database::get_recording_id_by_session(&session_id) {
        Ok(Some(id)) => id,
        Ok(None) => {
            eprintln!("Failed to find main recording for session: {}", session_id);
            0  // Use placeholder if not found
        },
        Err(e) => {
            eprintln!("Error getting recording ID from database: {}", e);
            0  // Use placeholder if error
        }
    };

    // Save recording segment metadata to database
    let segment_index = {
        let files_guard = RECORDING_SEGMENT_FILES.lock().unwrap();
        files_guard.len() - 1  // Current index is length - 1
    };

    if let Err(e) = database::save_recording_segment_to_db(
        recording_id,
        segment_index as i32,
        &format!("recording_{}_seg_{}.mkv", session_id, segment_index),
        Some(&video_path_str),
        None, // Duration not known yet
        None  // File size not known yet
    ) {
        eprintln!("Failed to save recording segment metadata to database: {}", e);
    }

    Ok(format!("Started new recording segment: {}", video_path_str))
}

#[tauri::command]
async fn pause_combined_recording(app: tauri::AppHandle) -> Result<String, String> {
    // Check if there's actually a recording in progress before pausing
    {
        let process_guard = COMBINED_RECORDING_PROCESS.lock().map_err(|e| e.to_string())?;
        if process_guard.is_none() {
            return Err("No recording in progress to pause".to_string());
        }
        // Don't drop the guard yet, just checking
    }

    // Stop the current recording segment
    stop_current_recording_segment().await?;

    // Set the paused flag
    RECORDING_PAUSED.store(true, Ordering::SeqCst);

    // Emit event to notify all UI windows
    // Emit to each active window
    for (_window_label, window) in app.webview_windows() {
        let _ = window.emit("recording-paused", "Recording has been paused");
    }

    Ok("Recording paused successfully - segment saved".to_string())
}

#[tauri::command]
async fn resume_combined_recording(app: tauri::AppHandle) -> Result<String, String> {
    // Check if there's a recording session but no active process (meaning it's paused)
    {
        let process_guard = COMBINED_RECORDING_PROCESS.lock().map_err(|e| e.to_string())?;
        if process_guard.is_some() {
            // If there's an active process, it means we're not paused
            return Err("Recording is not paused, cannot resume".to_string());
        }
        // Also check if we have a session ID to confirm we're in a recording session
        let session_guard = RECORDING_SESSION_ID.lock().unwrap();
        if session_guard.is_none() {
            return Err("No recording session is active".to_string());
        }
    }

    // Start a new recording segment
    let result = start_new_recording_segment().await?;

    // Clear the paused flag
    RECORDING_PAUSED.store(false, Ordering::SeqCst);

    // Emit event to notify all UI windows
    // Emit to each active window
    for (_window_label, window) in app.webview_windows() {
        let _ = window.emit("recording-resumed", "Recording has been resumed");
    }

    Ok(format!("Recording resumed successfully - {}", result))
}

#[tauri::command]
async fn get_network_stats() -> Result<String, String> {
    let stats = NETWORK_STATS.lock().unwrap();
    let duration = stats.last_updated.elapsed().as_secs_f64();

    // Calculate speeds (bytes per second)
    let download_speed = if duration > 0.0 {
        (stats.total_bytes_downloaded - stats.last_bytes_downloaded) as f64 / duration
    } else {
        0.0
    };
    let upload_speed = if duration > 0.0 {
        (stats.total_bytes_uploaded - stats.last_bytes_uploaded) as f64 / duration
    } else {
        0.0
    };

    // Convert to appropriate units (KB/s or MB/s)
    let download_speed_str = if download_speed > 1024.0 * 1024.0 {
        format!("{:.2} MB/s", download_speed / (1024.0 * 1024.0))
    } else {
        format!("{:.2} KB/s", download_speed / 1024.0)
    };

    let upload_speed_str = if upload_speed > 1024.0 * 1024.0 {
        format!("{:.2} MB/s", upload_speed / (1024.0 * 1024.0))
    } else {
        format!("{:.2} KB/s", upload_speed / 1024.0)
    };

    Ok(format!(r#"{{"downloadSpeed": "{}", "uploadSpeed": "{}", "totalDownloaded": "{}", "totalUploaded": "{}"}}"#,
        download_speed_str,
        upload_speed_str,
        format!("{:.2} MB", stats.total_bytes_downloaded as f64 / (1024.0 * 1024.0)),
        format!("{:.2} MB", stats.total_bytes_uploaded as f64 / (1024.0 * 1024.0))
    ))
}

#[tauri::command]
async fn get_global_network_stats() -> Result<String, String> {
    // Create a new Networks instance to get current network data
    let networks = Networks::new_with_refreshed_list();

    // Calculate total bytes across all network interfaces
    let mut total_bytes_downloaded = 0;
    let mut total_bytes_uploaded = 0;

    for (interface_name, network) in networks.iter() {
        // Skip loopback interfaces
        if interface_name.to_lowercase().contains("lo") || interface_name.to_lowercase().contains("loopback") {
            continue;
        }
        total_bytes_downloaded += network.total_received();
        total_bytes_uploaded += network.total_transmitted();
    }

    let mut global_stats = GLOBAL_NETWORK_STATS.lock().unwrap();
    let duration = global_stats.last_updated.elapsed().as_secs_f64();

    // Calculate speeds (bytes per second)
    let download_speed = if duration > 0.0 {
        (total_bytes_downloaded - global_stats.last_total_bytes_downloaded) as f64 / duration
    } else {
        0.0
    };

    let upload_speed = if duration > 0.0 {
        (total_bytes_uploaded - global_stats.last_total_bytes_uploaded) as f64 / duration
    } else {
        0.0
    };

    // Convert to appropriate units (KB/s or MB/s)
    let download_speed_str = if download_speed > 1024.0 * 1024.0 {
        format!("{:.2} MB/s", download_speed / (1024.0 * 1024.0))
    } else {
        format!("{:.2} KB/s", download_speed / 1024.0)
    };

    let upload_speed_str = if upload_speed > 1024.0 * 1024.0 {
        format!("{:.2} MB/s", upload_speed / (1024.0 * 1024.0))
    } else {
        format!("{:.2} KB/s", upload_speed / 1024.0)
    };

    // Update last values for next calculation
    global_stats.last_total_bytes_downloaded = total_bytes_downloaded;
    global_stats.last_total_bytes_uploaded = total_bytes_uploaded;
    global_stats.last_updated = std::time::Instant::now();

    Ok(format!(r#"{{"downloadSpeed": "{}", "uploadSpeed": "{}", "totalDownloaded": "{}", "totalUploaded": "{}"}}"#,
        download_speed_str,
        upload_speed_str,
        format!("{:.2} MB", total_bytes_downloaded as f64 / (1024.0 * 1024.0)),
        format!("{:.2} MB", total_bytes_uploaded as f64 / (1024.0 * 1024.0))
    ))
}

// Command to update network usage (would be called from download/upload operations)
#[tauri::command]
async fn update_network_usage(downloaded_bytes: u64, uploaded_bytes: u64) -> Result<String, String> {
    let mut stats = NETWORK_STATS.lock().unwrap();

    stats.total_bytes_downloaded += downloaded_bytes;
    stats.total_bytes_uploaded += uploaded_bytes;

    // Update last values and timestamp for speed calculation
    stats.last_bytes_downloaded = stats.total_bytes_downloaded;
    stats.last_bytes_uploaded = stats.total_bytes_uploaded;
    stats.last_updated = std::time::Instant::now();

    // Convert bytes to appropriate units for display
    let total_downloaded_mb = format!("{:.2} MB", stats.total_bytes_downloaded as f64 / (1024.0 * 1024.0));
    let total_uploaded_mb = format!("{:.2} MB", stats.total_bytes_uploaded as f64 / (1024.0 * 1024.0));

    // Calculate speeds (bytes per second)
    let duration = stats.last_updated.elapsed().as_secs_f64();
    let download_speed = if duration > 0.0 {
        (stats.total_bytes_downloaded - stats.last_bytes_downloaded) as f64 / duration
    } else {
        0.0
    };
    let upload_speed = if duration > 0.0 {
        (stats.total_bytes_uploaded - stats.last_bytes_uploaded) as f64 / duration
    } else {
        0.0
    };

    // Convert speeds to appropriate units
    let download_speed_str = if download_speed > 1024.0 * 1024.0 {
        format!("{:.2} MB/s", download_speed / (1024.0 * 1024.0))
    } else {
        format!("{:.2} KB/s", download_speed / 1024.0)
    };

    let upload_speed_str = if upload_speed > 1024.0 * 1024.0 {
        format!("{:.2} MB/s", upload_speed / (1024.0 * 1024.0))
    } else {
        format!("{:.2} KB/s", upload_speed / 1024.0)
    };

    // Save network usage to database
    if let Err(e) = database::save_network_usage_to_db(
        &download_speed_str,
        &upload_speed_str,
        &total_downloaded_mb,
        &total_uploaded_mb
    ) {
        eprintln!("Failed to save network usage to database: {}", e);
    }

    Ok("Network usage updated successfully".to_string())
}

#[tauri::command]
async fn get_screenshot_intervals() -> Result<String, String> {
    let min_interval = SCREENSHOT_MIN_INTERVAL.lock().unwrap();
    let max_interval = SCREENSHOT_MAX_INTERVAL.lock().unwrap();

    Ok(format!("{{\"min\": {}, \"max\": {}}}", *min_interval / 60, *max_interval / 60)) // Return in minutes
}

#[tauri::command]
async fn set_screenshot_intervals(min_minutes: u64, max_minutes: u64) -> Result<String, String> {
    if min_minutes >= max_minutes {
        return Err("Minimum interval must be less than maximum interval".to_string());
    }

    if min_minutes < 1 || max_minutes > 120 {
        return Err("Intervals must be between 1 and 120 minutes".to_string());
    }

    // Convert minutes to seconds
    let min_seconds = min_minutes * 60;
    let max_seconds = max_minutes * 60;

    {
        let mut min_guard = SCREENSHOT_MIN_INTERVAL.lock().unwrap();
        *min_guard = min_seconds;
    }

    {
        let mut max_guard = SCREENSHOT_MAX_INTERVAL.lock().unwrap();
        *max_guard = max_seconds;
    }

    Ok(format!("Screenshot intervals updated: min {} min, max {} min", min_minutes, max_minutes))
}

// Database retrieval commands for admin interface

#[tauri::command]
async fn get_screenshots_by_session(session_id: String) -> Result<String, String> {
    match database::get_screenshots_by_session(&session_id) {
        Ok(screenshots) => {
            match serde_json::to_string(&screenshots) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize screenshots: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to get screenshots from database: {}", e)),
    }
}

#[tauri::command]
async fn get_all_screenshots(limit: Option<u32>) -> Result<String, String> {
    match database::get_all_screenshots(limit) {
        Ok(screenshots) => {
            match serde_json::to_string(&screenshots) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize screenshots: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to get screenshots from database: {}", e)),
    }
}

#[tauri::command]
async fn get_recordings(limit: Option<u32>) -> Result<String, String> {
    match database::get_recordings(limit) {
        Ok(recordings) => {
            match serde_json::to_string(&recordings) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize recordings: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to get recordings from database: {}", e)),
    }
}

#[tauri::command]
async fn get_user_activity(limit: Option<u32>) -> Result<String, String> {
    match database::get_user_activity(limit) {
        Ok(activity) => {
            match serde_json::to_string(&activity) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize user activity: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to get user activity from database: {}", e)),
    }
}

#[tauri::command]
async fn get_network_usage(limit: Option<u32>) -> Result<String, String> {
    match database::get_network_usage(limit) {
        Ok(usage) => {
            match serde_json::to_string(&usage) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize network usage: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to get network usage from database: {}", e)),
    }
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
            stop_all_processes,
            get_process_status,
            update_user_activity,
            start_idle_detection,
            stop_idle_detection,
            add_excluded_window,
            remove_excluded_window,
            get_excluded_windows,
            create_admin_window,
            pause_combined_recording,
            resume_combined_recording,
            get_screenshot_intervals,
            set_screenshot_intervals,
            get_network_stats,
            get_global_network_stats,
            update_network_usage,
            get_screenshots_by_session,
            get_all_screenshots,
            get_recordings,
            get_user_activity,
            get_network_usage
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
