use mysql::*;
use mysql::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};
use lazy_static::lazy_static;

// Global flag to track if database is available
static DATABASE_AVAILABLE: AtomicBool = AtomicBool::new(true);

// Database connection pool - using lazy_static to initialize at runtime
lazy_static! {
    pub static ref DB_POOL: Option<Pool> = {
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

        match Pool::new(Opts::from_url(&url).expect("Invalid MySQL URL")) {
            Ok(pool) => {
                // Initialize database tables if they don't exist
                initialize_database(&pool);
                DATABASE_AVAILABLE.store(true, Ordering::SeqCst);
                Some(pool)
            },
            Err(e) => {
                eprintln!("Failed to create MySQL pool: {}", e);
                DATABASE_AVAILABLE.store(false, Ordering::SeqCst);
                None
            }
        }
    };
}

use std::sync::Mutex;
use std::time::{Duration, SystemTime};

// Track the last time we attempted to connect to the database
static LAST_CONNECT_ATTEMPT: Mutex<SystemTime> = Mutex::new(SystemTime::UNIX_EPOCH);

// Helper function to check if database is available with connection validation
pub fn is_database_available() -> bool {
    let current_status = DATABASE_AVAILABLE.load(Ordering::SeqCst);

    // If database is available according to our flag, check if connection is still valid
    if current_status {
        if let Some(ref pool) = *DB_POOL {
            if let Ok(mut conn) = pool.get_conn() {
                // Test the connection by executing a simple query
                let result: Option<u8> = conn.query_first("SELECT 1").unwrap_or(None);
                return result.is_some();
            } else {
                // If we can't get a connection, mark database as unavailable
                DATABASE_AVAILABLE.store(false, Ordering::SeqCst);
                return false;
            }
        } else {
            // Pool is not initialized
            DATABASE_AVAILABLE.store(false, Ordering::SeqCst);
            return false;
        }
    }

    // If database was not available, check if enough time has passed to try reconnection
    // Try to reconnect every 30 seconds
    if let Ok(last_attempt) = LAST_CONNECT_ATTEMPT.lock() {
        if let Ok(elapsed) = last_attempt.elapsed() {
            if elapsed > Duration::from_secs(30) {
                // Drop the lock before attempting to reconnect
                drop(last_attempt);

                // Try to reconnect by testing a new connection
                let db_config = DatabaseConfig::load();
                let url = format!(
                    "mysql://{}:{}@{}:{}/{}",
                    db_config.user,
                    db_config.password,
                    db_config.host,
                    db_config.port,
                    db_config.database
                );

                // Test if we can connect to the database now
                match Pool::new(Opts::from_url(&url).expect("Invalid MySQL URL")) {
                    Ok(test_pool) => {
                        // Test with a simple connection
                        if let Ok(mut conn) = test_pool.get_conn() {
                            let result: Option<u8> = conn.query_first("SELECT 1").unwrap_or(None);
                            if result.is_some() {
                                // The database is now available!
                                DATABASE_AVAILABLE.store(true, Ordering::SeqCst);

                                // Update the last connection attempt time
                                if let Ok(mut last_attempt) = LAST_CONNECT_ATTEMPT.lock() {
                                    *last_attempt = SystemTime::now();
                                }

                                println!("Database connection restored!");
                                return true;
                            }
                        }
                    },
                    Err(_) => {
                        // Failed to create test pool, database is still not available
                    }
                }

                // Update the last connection attempt time even on failure
                if let Ok(mut last_attempt) = LAST_CONNECT_ATTEMPT.lock() {
                    *last_attempt = SystemTime::now();
                }
            }
        }
    }

    DATABASE_AVAILABLE.load(Ordering::SeqCst)
}

// Helper function to attempt reconnection to database
fn try_reconnect_database() {
    let db_config = DatabaseConfig::load();

    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        db_config.user,
        db_config.password,
        db_config.host,
        db_config.port,
        db_config.database
    );

    match Pool::new(Opts::from_url(&url).expect("Invalid MySQL URL")) {
        Ok(pool) => {
            // Initialize database tables if they don't exist
            initialize_database(&pool);

            // The original DB_POOL is initialized with lazy_static and cannot be changed at runtime
            // But we can at least update the availability flag to reflect that connection is now possible
            DATABASE_AVAILABLE.store(true, Ordering::SeqCst);

            // Update the last connection attempt time
            if let Ok(mut last_attempt) = LAST_CONNECT_ATTEMPT.lock() {
                *last_attempt = SystemTime::now();
            }

            println!("Successfully reconnected to database!");
        },
        Err(e) => {
            eprintln!("Failed to reconnect to database: {}", e);

            // Update the last connection attempt time even on failure
            if let Ok(mut last_attempt) = LAST_CONNECT_ATTEMPT.lock() {
                *last_attempt = SystemTime::now();
            }
        }
    }
}

// Function to create a new user in the database
pub fn create_user(user_id: &str, username: Option<&str>, email: Option<&str>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // Log that database is not available but don't fail the operation
        eprintln!("Database not available, skipping user creation");
        return Ok(());
    }

    // Try to use the global pool, but if it's not available, try to create a direct connection
    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

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
    } else {
        // Try to connect directly if the global pool is not available
        let db_config = DatabaseConfig::load();
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            db_config.user,
            db_config.password,
            db_config.host,
            db_config.port,
            db_config.database
        );

        match Pool::new(Opts::from_url(&url).expect("Invalid MySQL URL")) {
            Ok(temp_pool) => {
                let mut conn = temp_pool.get_conn()?;
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

                // Update the global flag to indicate database is available
                DATABASE_AVAILABLE.store(true, Ordering::SeqCst);
            },
            Err(_) => {
                eprintln!("Unable to connect to database to create user");
            }
        }
    }

    Ok(())
}

// Function to get user by user_id
pub fn get_user(user_id: &str) -> Result<Option<UserInfo>, Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, return None
        eprintln!("Database not available, returning None for user query");
        return Ok(None);
    }

    let pool = DB_POOL.as_ref().ok_or("Database pool not available")?;
    let mut conn = pool.get_conn()?;

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
    if !is_database_available() {
        // If database is not available, assume user doesn't exist
        eprintln!("Database not available, assuming user doesn't exist");
        return Ok(false);
    }

    let pool = DB_POOL.as_ref().ok_or("Database pool not available")?;
    let mut conn = pool.get_conn()?;

    let result: Option<u32> = conn.exec_first(
        "SELECT id FROM users WHERE user_id = ?",
        (user_id,)
    )?;

    Ok(result.is_some())
}

// Function to get all users
pub fn get_all_users(limit: Option<u32>) -> Result<Vec<UserInfo>, Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, return an empty vector
        eprintln!("Database not available, returning empty user list");
        return Ok(Vec::new());
    }

    let pool = DB_POOL.as_ref().ok_or("Database pool not available")?;
    let mut conn = pool.get_conn()?;

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
    if let Err(e) = conn.query_drop(
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
    ) {
        eprintln!("Failed to create users table: {}", e);
    }

    // Create screenshots table
    if let Err(e) = conn.query_drop(
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
    ) {
        eprintln!("Failed to create screenshots table: {}", e);
    }

    // Create recordings table
    if let Err(e) = conn.query_drop(
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
    ) {
        eprintln!("Failed to create recordings table: {}", e);
    }

    // Create recording segments table
    if let Err(e) = conn.query_drop(
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
    ) {
        eprintln!("Failed to create recording_segments table: {}", e);
    }

    // Create user_activity table
    if let Err(e) = conn.query_drop(
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
    ) {
        eprintln!("Failed to create user_activity table: {}", e);
    }

    // Create network_usage table
    if let Err(e) = conn.query_drop(
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
    ) {
        eprintln!("Failed to create network_usage table: {}", e);
    }

    // Create excluded_windows table
    if let Err(e) = conn.query_drop(
        "CREATE TABLE IF NOT EXISTS excluded_windows (
            id INT AUTO_INCREMENT PRIMARY KEY,
            window_title VARCHAR(255) NOT NULL UNIQUE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )"
    ) {
        eprintln!("Failed to create excluded_windows table: {}", e);
    }

    // Create process_status table
    if let Err(e) = conn.query_drop(
        "CREATE TABLE IF NOT EXISTS process_status (
            id INT AUTO_INCREMENT PRIMARY KEY,
            recording_active BOOLEAN DEFAULT FALSE,
            screenshotting_active BOOLEAN DEFAULT FALSE,
            idle_detection_active BOOLEAN DEFAULT FALSE,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        )"
    ) {
        eprintln!("Failed to create process_status table: {}", e);
    }

    // Insert initial process status if not exists
    if let Err(e) = conn.query_drop(
        "INSERT IGNORE INTO process_status (recording_active, screenshotting_active, idle_detection_active)
         VALUES (FALSE, FALSE, FALSE)"
    ) {
        eprintln!("Failed to insert initial process status: {}", e);
    }
}

// Function to save screenshot metadata to database
pub fn save_screenshot_to_db(user_id: &str, session_id: &str, file_path: &str, filename: &str, file_size: Option<i64>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, try to connect directly
        let db_config = DatabaseConfig::load();
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            db_config.user,
            db_config.password,
            db_config.host,
            db_config.port,
            db_config.database
        );

        match Pool::new(Opts::from_url(&url).expect("Invalid MySQL URL")) {
            Ok(temp_pool) => {
                let mut conn = temp_pool.get_conn()?;

                // Ensure user exists in the users table
                create_user(user_id, None, None)?;

                // Insert screenshot record with detailed error handling
                if let Err(e) = conn.exec_drop(
                    "INSERT INTO screenshots (user_id, session_id, filename, file_path) VALUES (?, ?, ?, ?)",
                    (
                        user_id,
                        session_id,
                        filename,
                        file_path
                    )
                ) {
                    eprintln!("Failed to insert screenshot into database: {}", e);
                    return Err(Box::new(e));
                }

                // Update the global flag to indicate database is now available
                DATABASE_AVAILABLE.store(true, Ordering::SeqCst);
            },
            Err(_) => {
                eprintln!("Unable to connect to database to save screenshot metadata");
                // We're still returning Ok here to match the original behavior
                // The data just won't be saved to database if MySQL is not accessible
            }
        }
    } else {
        // If database is available via global pool, use it
        if let Some(ref pool) = *DB_POOL {
            let mut conn = pool.get_conn()?;

            // Ensure user exists in the users table
            create_user(user_id, None, None)?;

            // Insert screenshot record with detailed error handling
            if let Err(e) = conn.exec_drop(
                "INSERT INTO screenshots (user_id, session_id, filename, file_path) VALUES (?, ?, ?, ?)",
                (
                    user_id,
                    session_id,
                    filename,
                    file_path
                )
            ) {
                eprintln!("Failed to insert screenshot into database: {}", e);
                return Err(Box::new(e));
            }
        } else {
            eprintln!("Database pool is not available");
            return Err("Database pool is not available".into());
        }
    }

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
    if !is_database_available() {
        // If database is not available, try to connect directly
        let db_config = DatabaseConfig::load();
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            db_config.user,
            db_config.password,
            db_config.host,
            db_config.port,
            db_config.database
        );

        match Pool::new(Opts::from_url(&url).expect("Invalid MySQL URL")) {
            Ok(temp_pool) => {
                let mut conn = temp_pool.get_conn()?;

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
                // Update the global flag to indicate database is now available
                DATABASE_AVAILABLE.store(true, Ordering::SeqCst);
                Ok(id.unwrap_or(0))
            },
            Err(_) => {
                eprintln!("Unable to connect to database to save recording metadata");
                // Return a placeholder ID to match the original behavior
                Ok(0)
            }
        }
    } else {
        // If database is available via global pool, use it
        if let Some(ref pool) = *DB_POOL {
            let mut conn = pool.get_conn()?;

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
        } else {
            eprintln!("Database pool is not available");
            Ok(0)
        }
    }
}

// Function to get recording ID by session ID
pub fn get_recording_id_by_session(session_id: &str) -> Result<Option<u64>, Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, return None
        eprintln!("Database not available, returning None for recording ID query");
        return Ok(None);
    }

    let pool = DB_POOL.as_ref().unwrap();
    let mut conn = pool.get_conn()?;

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
    if !is_database_available() {
        // If database is not available, log and continue
        eprintln!("Database not available, skipping recording segment save");
        return Ok(());
    }

    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

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
    } else {
        eprintln!("Database pool is not available");
    }

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
    if !is_database_available() {
        // If database is not available, log and continue
        eprintln!("Database not available, skipping recording metadata update");
        return Ok(());
    }

    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

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
    } else {
        eprintln!("Database pool is not available");
    }

    Ok(())
}

// Function to save user activity to database
pub fn save_user_activity_to_db(user_id: &str, activity_type: &str, duration_seconds: Option<i32>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, log and continue
        eprintln!("Database not available, skipping user activity save");
        return Ok(());
    }

    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

        // Ensure user exists in the users table
        create_user(user_id, None, None)?;

        conn.exec_drop(
            "INSERT INTO user_activity (user_id, activity_type, duration_seconds) VALUES (?, ?, ?)",
            (user_id, activity_type, duration_seconds.unwrap_or(0))
        )?;
    } else {
        eprintln!("Database pool is not available");
    }

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
    if !is_database_available() {
        // If database is not available, try to connect directly
        let db_config = DatabaseConfig::load();
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            db_config.user,
            db_config.password,
            db_config.host,
            db_config.port,
            db_config.database
        );

        match Pool::new(Opts::from_url(&url).expect("Invalid MySQL URL")) {
            Ok(temp_pool) => {
                let mut conn = temp_pool.get_conn()?;

                // Ensure user exists in the users table
                create_user(user_id, None, None)?;

                conn.exec_drop(
                    "INSERT INTO network_usage (user_id, download_speed, upload_speed, total_downloaded, total_uploaded) VALUES (?, ?, ?, ?, ?)",
                    (user_id, download_speed, upload_speed, total_downloaded, total_uploaded)
                )?;

                // Update the global flag to indicate database is now available
                DATABASE_AVAILABLE.store(true, Ordering::SeqCst);
            },
            Err(_) => {
                eprintln!("Unable to connect to database to save network usage");
            }
        }
    } else {
        // If database is available via global pool, use it
        if let Some(ref pool) = *DB_POOL {
            let mut conn = pool.get_conn()?;

            // Ensure user exists in the users table
            create_user(user_id, None, None)?;

            conn.exec_drop(
                "INSERT INTO network_usage (user_id, download_speed, upload_speed, total_downloaded, total_uploaded) VALUES (?, ?, ?, ?, ?)",
                (user_id, download_speed, upload_speed, total_downloaded, total_uploaded)
            )?;
        } else {
            eprintln!("Database pool is not available");
        }
    }

    Ok(())
}

// Function to add excluded window to database
pub fn add_excluded_window_to_db(window_title: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, log and continue
        eprintln!("Database not available, skipping excluded window addition");
        return Ok(());
    }

    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

        conn.exec_drop(
            "INSERT IGNORE INTO excluded_windows (window_title) VALUES (?)",
            (window_title,)
        )?;
    } else {
        eprintln!("Database pool is not available");
    }

    Ok(())
}

// Function to remove excluded window from database
pub fn remove_excluded_window_from_db(window_title: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, log and continue
        eprintln!("Database not available, skipping excluded window removal");
        return Ok(());
    }

    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

        conn.exec_drop(
            "DELETE FROM excluded_windows WHERE window_title = ?",
            (window_title,)
        )?;
    } else {
        eprintln!("Database pool is not available");
    }

    Ok(())
}

// Function to get all excluded windows from database
pub fn get_excluded_windows_from_db() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, return an empty vector
        eprintln!("Database not available, returning empty excluded windows list");
        return Ok(Vec::new());
    }

    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

        let result: Vec<String> = conn
            .query_map("SELECT window_title FROM excluded_windows", |window_title| {
                window_title
            })?;

        Ok(result)
    } else {
        eprintln!("Database pool is not available");
        Ok(Vec::new())
    }
}

// Function to update process status in database
pub fn update_process_status_in_db(
    recording_active: bool,
    screenshotting_active: bool,
    idle_detection_active: bool
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, log and continue
        eprintln!("Database not available, skipping process status update");
        return Ok(());
    }

    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

        conn.exec_drop(
            "UPDATE process_status SET recording_active = ?, screenshotting_active = ?, idle_detection_active = ?, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
            (recording_active, screenshotting_active, idle_detection_active)
        )?;
    } else {
        eprintln!("Database pool is not available");
    }

    Ok(())
}

// Function to get screenshots by session ID from database
pub fn get_screenshots_by_session(user_id: &str, session_id: &str) -> Result<Vec<ScreenshotData>, Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, return an empty vector
        eprintln!("Database not available, returning empty screenshot list");
        return Ok(Vec::new());
    }

    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

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
    } else {
        eprintln!("Database pool is not available");
        Ok(Vec::new())
    }
}

// Function to get all screenshots from database for a specific user
pub fn get_all_screenshots(user_id: &str, limit: Option<u32>) -> Result<Vec<ScreenshotData>, Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, return an empty vector
        eprintln!("Database not available, returning empty screenshot list");
        return Ok(Vec::new());
    }

    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

        if let Some(lim) = limit {
            let result = conn.exec_map(
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
            )?;
            Ok(result)
        } else {
            let result = conn.exec_map(
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
            )?;
            Ok(result)
        }
    } else {
        eprintln!("Database pool is not available");
        Ok(Vec::new())
    }
}

// Function to get recordings from database for a specific user
pub fn get_recordings(user_id: &str, limit: Option<u32>) -> Result<Vec<RecordingData>, Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, return an empty vector
        eprintln!("Database not available, returning empty recording list");
        return Ok(Vec::new());
    }

    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

        if let Some(lim) = limit {
            let result = conn.exec_map(
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
            )?;
            Ok(result)
        } else {
            let result = conn.exec_map(
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
            )?;
            Ok(result)
        }
    } else {
        eprintln!("Database pool is not available");
        Ok(Vec::new())
    }
}

// Function to get user activity from database for a specific user
pub fn get_user_activity(user_id: &str, limit: Option<u32>) -> Result<Vec<UserActivityData>, Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, return an empty vector
        eprintln!("Database not available, returning empty user activity list");
        return Ok(Vec::new());
    }

    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

        if let Some(lim) = limit {
            let result = conn.exec_map(
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
            )?;
            Ok(result)
        } else {
            let result = conn.exec_map(
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
            )?;
            Ok(result)
        }
    } else {
        eprintln!("Database pool is not available");
        Ok(Vec::new())
    }
}

// Function to get network usage from database for a specific user
pub fn get_network_usage(user_id: &str, limit: Option<u32>) -> Result<Vec<NetworkUsageData>, Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, return an empty vector
        eprintln!("Database not available, returning empty network usage list");
        return Ok(Vec::new());
    }

    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

        if let Some(lim) = limit {
            let result = conn.exec_map(
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
            )?;
            Ok(result)
        } else {
            let result = conn.exec_map(
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
            )?;
            Ok(result)
        }
    } else {
        eprintln!("Database pool is not available");
        Ok(Vec::new())
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