use mysql::*;
use mysql::prelude::*;
use std::sync::Arc;
use lazy_static::lazy_static;

// Database connection pool
lazy_static! {
    pub static ref DB_POOL: Arc<Pool> = {
        // Try environment variables first, then use config file, then defaults
        let db_config = DatabaseConfig::load();

        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            db_config.user,
            db_config.password,
            db_config.host,
            db_config.port,
            db_config.database
        );

        let opts = Opts::from_url(&url).expect("Invalid MySQL URL");
        let pool = Pool::new(opts).expect("Failed to create MySQL pool");

        // Initialize database tables if they don't exist
        initialize_database(&pool);

        Arc::new(pool)
    };
}

// Function to create a new user in the database
pub fn create_user(user_id: &str, username: Option<&str>, email: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    conn.exec_drop(
        "INSERT INTO users (user_id, username, email) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE username = COALESCE(?, username), email = COALESCE(?, email)",
        (
            user_id,
            username.unwrap_or(""),
            email.unwrap_or(""),
            username.unwrap_or(""),
            email.unwrap_or("")
        )
    )?;

    Ok(())
}

// Function to get user by user_id
pub fn get_user(user_id: &str) -> Result<Option<UserInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    let result: Option<UserInfo> = conn
        .exec_first(
            "SELECT id, user_id, username, email, created_at, updated_at, is_active FROM users WHERE user_id = ?",
            (user_id,)
        )?
        .map(|(id, db_user_id, username, email, created_at, updated_at, is_active): (u32, String, Option<String>, Option<String>, String, String, bool)| {
            UserInfo {
                id,
                user_id: db_user_id,
                username,
                email,
                created_at,
                updated_at,
                is_active,
            }
        });

    Ok(result)
}

// Function to check if a user exists
pub fn user_exists(user_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    let result: Option<u32> = conn.exec_first(
        "SELECT id FROM users WHERE user_id = ?",
        (user_id,)
    )?;

    Ok(result.is_some())
}

// Function to get all users
pub fn get_all_users(limit: Option<u32>) -> Result<Vec<UserInfo>, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    if let Some(lim) = limit {
        Ok(conn.exec_map(
            "SELECT id, user_id, username, email, created_at, updated_at, is_active FROM users ORDER BY created_at DESC LIMIT ?",
            (lim,),
            |(id, user_id, username, email, created_at, updated_at, is_active): (u32, String, Option<String>, Option<String>, String, String, bool)| {
                UserInfo {
                    id,
                    user_id,
                    username,
                    email,
                    created_at,
                    updated_at,
                    is_active,
                }
            }
        )?)
    } else {
        Ok(conn.exec_map(
            "SELECT id, user_id, username, email, created_at, updated_at, is_active FROM users ORDER BY created_at DESC",
            (),
            |(id, user_id, username, email, created_at, updated_at, is_active): (u32, String, Option<String>, Option<String>, String, String, bool)| {
                UserInfo {
                    id,
                    user_id,
                    username,
                    email,
                    created_at,
                    updated_at,
                    is_active,
                }
            }
        )?)
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub database: String,
}

impl DatabaseConfig {
    pub fn load() -> Self {
        // First try environment variables
        let user = std::env::var("MYSQL_USER").unwrap_or_else(|_| "root".to_string());
        let password = std::env::var("MYSQL_PASSWORD").unwrap_or_else(|_| "".to_string());
        let host = std::env::var("MYSQL_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("MYSQL_PORT").unwrap_or_else(|_| "3306".to_string());
        let database = std::env::var("MYSQL_DATABASE").unwrap_or_else(|_| "remote_work_db".to_string());

        DatabaseConfig {
            user,
            password,
            host,
            port,
            database,
        }
    }

    pub fn with_defaults() -> Self {
        DatabaseConfig {
            user: "root".to_string(),
            password: "".to_string(),
            host: "localhost".to_string(),
            port: "3306".to_string(),
            database: "remote_work_db".to_string(),
        }
    }
}

// Initialize database tables
fn initialize_database(pool: &Pool) {
    let mut conn = pool.get_conn().expect("Failed to get database connection");

    // Create users table first (since other tables reference it)
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL UNIQUE,
            username VARCHAR(255),
            email VARCHAR(255),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            is_active BOOLEAN DEFAULT TRUE,
            INDEX idx_user_id (user_id)
        )"
    ).expect("Failed to create users table");

    // Create screenshots table
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS screenshots (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            session_id VARCHAR(255) NOT NULL,
            image_data LONGBLOB,
            filename VARCHAR(255) NOT NULL,
            file_path VARCHAR(500),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
            INDEX idx_user_id (user_id),
            INDEX idx_session_id (session_id),
            INDEX idx_created_at (created_at)
        )"
    ).expect("Failed to create screenshots table");

    // Create recordings table
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS recordings (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            session_id VARCHAR(255) NOT NULL,
            filename VARCHAR(255) NOT NULL,
            file_path VARCHAR(500),
            duration_seconds INT,
            file_size BIGINT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
            INDEX idx_user_id (user_id),
            INDEX idx_session_id (session_id),
            INDEX idx_created_at (created_at)
        )"
    ).expect("Failed to create recordings table");

    // Create recording segments table
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS recording_segments (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            recording_id INT,
            segment_number INT NOT NULL,
            filename VARCHAR(255) NOT NULL,
            file_path VARCHAR(500),
            duration_seconds INT,
            file_size BIGINT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
            FOREIGN KEY (recording_id) REFERENCES recordings(id) ON DELETE CASCADE,
            INDEX idx_user_id (user_id),
            INDEX idx_recording_id (recording_id)
        )"
    ).expect("Failed to create recording_segments table");

    // Create user_activity table
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS user_activity (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            activity_type ENUM('active', 'idle') NOT NULL,
            duration_seconds INT,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
            INDEX idx_user_id (user_id),
            INDEX idx_timestamp (timestamp)
        )"
    ).expect("Failed to create user_activity table");

    // Create network_usage table
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS network_usage (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_id VARCHAR(255) NOT NULL,
            download_speed VARCHAR(50),
            upload_speed VARCHAR(50),
            total_downloaded VARCHAR(50),
            total_uploaded VARCHAR(50),
            recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
            INDEX idx_user_id (user_id),
            INDEX idx_recorded_at (recorded_at)
        )"
    ).expect("Failed to create network_usage table");
    
    // Create excluded_windows table
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS excluded_windows (
            id INT AUTO_INCREMENT PRIMARY KEY,
            window_title VARCHAR(255) NOT NULL UNIQUE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ).expect("Failed to create excluded_windows table");
    
    // Create process_status table
    conn.query_drop(
        "CREATE TABLE IF NOT EXISTS process_status (
            id INT AUTO_INCREMENT PRIMARY KEY,
            recording_active BOOLEAN DEFAULT FALSE,
            screenshotting_active BOOLEAN DEFAULT FALSE,
            idle_detection_active BOOLEAN DEFAULT FALSE,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        )"
    ).expect("Failed to create process_status table");
    
    // Insert initial process status if not exists
    conn.query_drop(
        "INSERT IGNORE INTO process_status (recording_active, screenshotting_active, idle_detection_active) 
         VALUES (FALSE, FALSE, FALSE)"
    ).expect("Failed to insert initial process status");
}

// Function to save screenshot metadata to database
pub fn save_screenshot_to_db(user_id: &str, session_id: &str, file_path: &str, filename: &str, file_size: Option<i64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    // Ensure user exists in the users table
    create_user(user_id, None, None)?;

    conn.exec_drop(
        "INSERT INTO screenshots (user_id, session_id, file_path, filename, file_size) VALUES (?, ?, ?, ?, ?)",
        (
            user_id,
            session_id,
            file_path,
            filename,
            file_size.unwrap_or(0)
        )
    )?;

    Ok(())
}

// Function to save recording metadata to database
pub fn save_recording_to_db(
    user_id: &str,
    session_id: &str,
    filename: &str,
    file_path: Option<&str>,
    duration_seconds: Option<i32>,
    file_size: Option<i64>
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    // Ensure user exists in the users table
    create_user(user_id, None, None)?;

    conn.exec_drop(
        "INSERT INTO recordings (user_id, session_id, filename, file_path, duration_seconds, file_size) VALUES (?, ?, ?, ?, ?, ?)",
        (
            user_id,
            session_id,
            filename,
            file_path.unwrap_or(""),
            duration_seconds.unwrap_or(0),
            file_size.unwrap_or(0)
        )
    )?;

    // Get the ID of the inserted recording
    let id: Option<u64> = conn.exec_first("SELECT LAST_INSERT_ID()", ())?;
    Ok(id.unwrap_or(0))
}

// Function to get recording ID by session ID
pub fn get_recording_id_by_session(session_id: &str) -> Result<Option<u64>, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    let result: Option<u64> = conn.exec_first(
        "SELECT id FROM recordings WHERE session_id = ? ORDER BY id DESC LIMIT 1",
        (session_id,)
    )?;

    Ok(result)
}

// Function to save recording segment to database
pub fn save_recording_segment_to_db(
    user_id: &str,
    recording_id: u64,
    segment_number: i32,
    filename: &str,
    file_path: Option<&str>,
    duration_seconds: Option<i32>,
    file_size: Option<i64>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    // Ensure user exists in the users table
    create_user(user_id, None, None)?;

    conn.exec_drop(
        "INSERT INTO recording_segments (user_id, recording_id, segment_number, filename, file_path, duration_seconds, file_size) VALUES (?, ?, ?, ?, ?, ?, ?)",
        (
            user_id,
            recording_id,
            segment_number,
            filename,
            file_path.unwrap_or(""),
            duration_seconds.unwrap_or(0),
            file_size.unwrap_or(0)
        )
    )?;

    Ok(())
}

// Function to update recording metadata in database after completion
pub fn update_recording_metadata_in_db(
    session_id: &str,
    final_filename: Option<&str>,
    final_file_path: Option<&str>,
    duration_seconds: Option<i32>,
    file_size: Option<i64>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    let query = if final_file_path.is_some() && final_filename.is_some() {
        "UPDATE recordings SET filename = ?, file_path = ?, duration_seconds = ?, file_size = ? WHERE session_id = ?"
    } else if final_file_path.is_some() {
        "UPDATE recordings SET file_path = ?, duration_seconds = ?, file_size = ? WHERE session_id = ?"
    } else if final_filename.is_some() {
        "UPDATE recordings SET filename = ?, duration_seconds = ?, file_size = ? WHERE session_id = ?"
    } else {
        "UPDATE recordings SET duration_seconds = ?, file_size = ? WHERE session_id = ?"
    };

    let result = if final_file_path.is_some() && final_filename.is_some() {
        conn.exec_drop(
            query,
            (
                final_filename.unwrap(),
                final_file_path.unwrap(),
                duration_seconds.unwrap_or(0),
                file_size.unwrap_or(0),
                session_id
            )
        )
    } else if final_file_path.is_some() {
        conn.exec_drop(
            query,
            (
                final_file_path.unwrap(),
                duration_seconds.unwrap_or(0),
                file_size.unwrap_or(0),
                session_id
            )
        )
    } else if final_filename.is_some() {
        conn.exec_drop(
            query,
            (
                final_filename.unwrap(),
                duration_seconds.unwrap_or(0),
                file_size.unwrap_or(0),
                session_id
            )
        )
    } else {
        conn.exec_drop(
            query,
            (
                duration_seconds.unwrap_or(0),
                file_size.unwrap_or(0),
                session_id
            )
        )
    };

    result?;
    Ok(())
}

// Function to save user activity to database
pub fn save_user_activity_to_db(user_id: &str, activity_type: &str, duration_seconds: Option<i32>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    // Ensure user exists in the users table
    create_user(user_id, None, None)?;

    conn.exec_drop(
        "INSERT INTO user_activity (user_id, activity_type, duration_seconds) VALUES (?, ?, ?)",
        (user_id, activity_type, duration_seconds.unwrap_or(0))
    )?;

    Ok(())
}

// Function to save network usage to database
pub fn save_network_usage_to_db(
    user_id: &str,
    download_speed: &str,
    upload_speed: &str,
    total_downloaded: &str,
    total_uploaded: &str
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    // Ensure user exists in the users table
    create_user(user_id, None, None)?;

    conn.exec_drop(
        "INSERT INTO network_usage (user_id, download_speed, upload_speed, total_downloaded, total_uploaded) VALUES (?, ?, ?, ?, ?)",
        (user_id, download_speed, upload_speed, total_downloaded, total_uploaded)
    )?;

    Ok(())
}

// Function to add excluded window to database
pub fn add_excluded_window_to_db(window_title: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;
    
    conn.exec_drop(
        "INSERT IGNORE INTO excluded_windows (window_title) VALUES (?)",
        (window_title,)
    )?;
    
    Ok(())
}

// Function to remove excluded window from database
pub fn remove_excluded_window_from_db(window_title: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;
    
    conn.exec_drop(
        "DELETE FROM excluded_windows WHERE window_title = ?",
        (window_title,)
    )?;
    
    Ok(())
}

// Function to get all excluded windows from database
pub fn get_excluded_windows_from_db() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;
    
    let result: Vec<String> = conn
        .query_map("SELECT window_title FROM excluded_windows", |window_title| {
            window_title
        })?;
    
    Ok(result)
}

// Function to update process status in database
pub fn update_process_status_in_db(
    recording_active: bool,
    screenshotting_active: bool,
    idle_detection_active: bool
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    conn.exec_drop(
        "UPDATE process_status SET recording_active = ?, screenshotting_active = ?, idle_detection_active = ?, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        (recording_active, screenshotting_active, idle_detection_active)
    )?;

    Ok(())
}

// Function to get screenshots by session ID from database
pub fn get_screenshots_by_session(user_id: &str, session_id: &str) -> Result<Vec<ScreenshotData>, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    let result: Vec<ScreenshotData> = conn
        .exec_map(
            "SELECT id, session_id, file_path, filename, file_size, created_at FROM screenshots WHERE user_id = ? AND session_id = ? ORDER BY created_at DESC",
            (user_id, session_id),
            |(id, session_id_db, file_path, filename, file_size, created_at): (u32, String, String, String, Option<i64>, String)| {
                ScreenshotData {
                    id,
                    session_id: session_id_db,
                    file_path,
                    filename,
                    file_size,
                    created_at,
                }
            }
        )?;

    Ok(result)
}

// Function to get all screenshots from database for a specific user
pub fn get_all_screenshots(user_id: &str, limit: Option<u32>) -> Result<Vec<ScreenshotData>, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    if let Some(lim) = limit {
        Ok(conn.exec_map(
            "SELECT id, session_id, file_path, filename, file_size, created_at FROM screenshots WHERE user_id = ? ORDER BY created_at DESC LIMIT ?",
            (user_id, lim),
            |(id, session_id, file_path, filename, file_size, created_at): (u32, String, String, String, Option<i64>, String)| {
                ScreenshotData {
                    id,
                    session_id,
                    file_path,
                    filename,
                    file_size,
                    created_at,
                }
            }
        )?)
    } else {
        Ok(conn.exec_map(
            "SELECT id, session_id, file_path, filename, file_size, created_at FROM screenshots WHERE user_id = ? ORDER BY created_at DESC",
            (user_id,),
            |(id, session_id, file_path, filename, file_size, created_at): (u32, String, String, String, Option<i64>, String)| {
                ScreenshotData {
                    id,
                    session_id,
                    file_path,
                    filename,
                    file_size,
                    created_at,
                }
            }
        )?)
    }
}

// Function to get recordings from database for a specific user
pub fn get_recordings(user_id: &str, limit: Option<u32>) -> Result<Vec<RecordingData>, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    if let Some(lim) = limit {
        Ok(conn.exec_map(
            "SELECT id, session_id, filename, file_path, duration_seconds, file_size, created_at FROM recordings WHERE user_id = ? ORDER BY created_at DESC LIMIT ?",
            (user_id, lim),
            |(id, session_id, filename, file_path, duration_seconds, file_size, created_at): (u32, String, String, String, i32, i64, String)| {
                RecordingData {
                    id,
                    session_id,
                    filename,
                    file_path,
                    duration_seconds,
                    file_size,
                    created_at,
                }
            }
        )?)
    } else {
        Ok(conn.exec_map(
            "SELECT id, session_id, filename, file_path, duration_seconds, file_size, created_at FROM recordings WHERE user_id = ? ORDER BY created_at DESC",
            (user_id,),
            |(id, session_id, filename, file_path, duration_seconds, file_size, created_at): (u32, String, String, String, i32, i64, String)| {
                RecordingData {
                    id,
                    session_id,
                    filename,
                    file_path,
                    duration_seconds,
                    file_size,
                    created_at,
                }
            }
        )?)
    }
}

// Function to get user activity from database for a specific user
pub fn get_user_activity(user_id: &str, limit: Option<u32>) -> Result<Vec<UserActivityData>, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    if let Some(lim) = limit {
        Ok(conn.exec_map(
            "SELECT id, activity_type, duration_seconds, timestamp FROM user_activity WHERE user_id = ? ORDER BY timestamp DESC LIMIT ?",
            (user_id, lim),
            |(id, activity_type, duration_seconds, timestamp): (u32, String, i32, String)| {
                UserActivityData {
                    id,
                    activity_type,
                    duration_seconds,
                    timestamp,
                }
            }
        )?)
    } else {
        Ok(conn.exec_map(
            "SELECT id, activity_type, duration_seconds, timestamp FROM user_activity WHERE user_id = ? ORDER BY timestamp DESC",
            (user_id,),
            |(id, activity_type, duration_seconds, timestamp): (u32, String, i32, String)| {
                UserActivityData {
                    id,
                    activity_type,
                    duration_seconds,
                    timestamp,
                }
            }
        )?)
    }
}

// Function to get network usage from database for a specific user
pub fn get_network_usage(user_id: &str, limit: Option<u32>) -> Result<Vec<NetworkUsageData>, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = DB_POOL.get_conn()?;

    if let Some(lim) = limit {
        Ok(conn.exec_map(
            "SELECT id, download_speed, upload_speed, total_downloaded, total_uploaded, recorded_at FROM network_usage WHERE user_id = ? ORDER BY recorded_at DESC LIMIT ?",
            (user_id, lim),
            |(id, download_speed, upload_speed, total_downloaded, total_uploaded, recorded_at): (u32, String, String, String, String, String)| {
                NetworkUsageData {
                    id,
                    download_speed,
                    upload_speed,
                    total_downloaded,
                    total_uploaded,
                    recorded_at,
                }
            }
        )?)
    } else {
        Ok(conn.exec_map(
            "SELECT id, download_speed, upload_speed, total_downloaded, total_uploaded, recorded_at FROM network_usage WHERE user_id = ? ORDER BY recorded_at DESC",
            (user_id,),
            |(id, download_speed, upload_speed, total_downloaded, total_uploaded, recorded_at): (u32, String, String, String, String, String)| {
                NetworkUsageData {
                    id,
                    download_speed,
                    upload_speed,
                    total_downloaded,
                    total_uploaded,
                    recorded_at,
                }
            }
        )?)
    }
}

// Data structure for user information
#[derive(Debug, serde::Serialize)]
pub struct UserInfo {
    pub id: u32,
    pub user_id: String,
    pub username: Option<String>,
    pub email: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub is_active: bool,
}

// Data structures for returning data from database
#[derive(Debug, serde::Serialize)]
pub struct ScreenshotData {
    pub id: u32,
    pub session_id: String,
    pub file_path: String,
    pub filename: String,
    pub file_size: Option<i64>,
    pub created_at: String,  // Using String as it's coming from SQL TIMESTAMP
}

#[derive(Debug, serde::Serialize)]
pub struct RecordingData {
    pub id: u32,
    pub session_id: String,
    pub filename: String,
    pub file_path: String,
    pub duration_seconds: i32,
    pub file_size: i64,
    pub created_at: String,
}

#[derive(Debug, serde::Serialize)]
pub struct UserActivityData {
    pub id: u32,
    pub activity_type: String, // 'active' or 'idle'
    pub duration_seconds: i32,
    pub timestamp: String,
}

#[derive(Debug, serde::Serialize)]
pub struct NetworkUsageData {
    pub id: u32,
    pub download_speed: String,
    pub upload_speed: String,
    pub total_downloaded: String,
    pub total_uploaded: String,
    pub recorded_at: String,
}