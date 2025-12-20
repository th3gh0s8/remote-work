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

        // Just update the username and email if the RepID already exists
        // This approach avoids issues with required fields in the salesrep table
        conn.exec_drop(
            "UPDATE salesrep SET username = COALESCE(?, username), repMail = COALESCE(?, repMail) WHERE RepID = ?",
            (
                username.unwrap_or(""),
                email.unwrap_or(""),
                user_id
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
                // Just update the username and email if the RepID already exists
                // This approach avoids issues with required fields in the salesrep table
                conn.exec_drop(
                    "UPDATE salesrep SET username = COALESCE(?, username), repMail = COALESCE(?, repMail) WHERE RepID = ?",
                    (
                        username.unwrap_or(""),
                        email.unwrap_or(""),
                        user_id
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

// Function to get user by user_id (from salesrep table)
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
            "SELECT ID, RepID, username, repMail, recordDate, recordTime, Actives FROM salesrep WHERE RepID = ?",
            (user_id,)
        )?
        .map(|(id, db_user_id, username, email, created_at, updated_at, is_active): (u32, String, Option<String>, Option<String>, String, String, String)| {
            UserInfo {
                id,
                user_id: db_user_id,
                username,
                email,
                created_at,
                updated_at: updated_at,
                is_active: is_active == "YES",
            }
        });

    Ok(result)
}

// Function to check if a user exists (in salesrep table)
pub fn user_exists(user_id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    if !is_database_available() {
        // If database is not available, assume user doesn't exist
        eprintln!("Database not available, assuming user doesn't exist");
        return Ok(false);
    }

    let pool = DB_POOL.as_ref().ok_or("Database pool not available")?;
    let mut conn = pool.get_conn()?;

    let result: Option<u32> = conn.exec_first(
        "SELECT ID FROM salesrep WHERE RepID = ?",
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
            "SELECT ID, RepID, username, repMail, recordDate, recordTime, Actives FROM salesrep ORDER BY recordDate DESC, recordTime DESC LIMIT ?",
            (lim,),
            |(id, user_id, username, email, created_at, updated_at, is_active): (u32, String, Option<String>, Option<String>, String, String, String)| {
                UserInfo {
                    id,
                    user_id,
                    username,
                    email,
                    created_at,
                    updated_at: updated_at,
                    is_active: is_active == "YES",
                }
            }
        )?)
    } else {
        Ok(conn.exec_map(
            "SELECT ID, RepID, username, repMail, recordDate, recordTime, Actives FROM salesrep ORDER BY recordDate DESC, recordTime DESC",
            (),
            |(id, user_id, username, email, created_at, updated_at, is_active): (u32, String, Option<String>, Option<String>, String, String, String)| {
                UserInfo {
                    id,
                    user_id,
                    username,
                    email,
                    created_at,
                    updated_at: updated_at,
                    is_active: is_active == "YES",
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
        let database = std::env::var("MYSQL_DATABASE").unwrap_or_else(|_| "remote-xwork".to_string());

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
            database: "remote-xwork".to_string(),
        }
    }
}

// Initialize database tables
fn initialize_database(pool: &Pool) {
    let mut conn = pool.get_conn().expect("Failed to get database connection");

    // Note: Only existing tables in remote-xwork database are used
    // The application will adapt to use the existing schema
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

                // Get the salesrep ID (the primary key) from the RepID
                let salesrep_id: Option<u32> = conn.exec_first(
                    "SELECT ID FROM salesrep WHERE RepID = ?",
                    (user_id,)
                )?;

                if let Some(id) = salesrep_id {
                    // Insert screenshot record into the web_images table which exists in remote-xwork
                    if let Err(e) = conn.exec_drop(
                        "INSERT INTO web_images (br_id, imgID, imgName, itmName, type, user_id, date, time, status) VALUES (?, ?, ?, ?, ?, ?, CURDATE(), CURTIME(), 'active')",
                        (
                            1, // Default br_id
                            0, // imgID - using 0 as default
                            filename,
                            session_id, // Use session_id as item name
                            "screenshot", // type
                            id, // user_id
                        )
                    ) {
                        eprintln!("Failed to insert screenshot into web_images table: {}", e);
                        return Err(Box::new(e));
                    }
                } else {
                    eprintln!("User with RepID {} not found in salesrep table", user_id);
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

            // Get the salesrep ID (the primary key) from the RepID
            let salesrep_id: Option<u32> = conn.exec_first(
                "SELECT ID FROM salesrep WHERE RepID = ?",
                (user_id,)
            )?;

            if let Some(id) = salesrep_id {
                // Insert screenshot record into the web_images table which exists in remote-xwork
                if let Err(e) = conn.exec_drop(
                    "INSERT INTO web_images (br_id, imgID, imgName, itmName, type, user_id, date, time, status) VALUES (?, ?, ?, ?, ?, ?, CURDATE(), CURTIME(), 'active')",
                    (
                        1, // Default br_id
                        0, // imgID - using 0 as default
                        filename,
                        session_id, // Use session_id as item name
                        "screenshot", // type
                        id, // user_id
                    )
                ) {
                    eprintln!("Failed to insert screenshot into web_images table: {}", e);
                    return Err(Box::new(e));
                }
            } else {
                eprintln!("User with RepID {} not found in salesrep table", user_id);
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

                // Get the salesrep ID (the primary key) from the RepID
                let salesrep_id: Option<u32> = conn.exec_first(
                    "SELECT ID FROM salesrep WHERE RepID = ?",
                    (user_id,)
                )?;

                if let Some(id) = salesrep_id {
                    conn.exec_drop(
                        "INSERT INTO web_images (br_id, imgID, imgName, itmName, type, user_id, date, time, status) VALUES (?, ?, ?, ?, ?, ?, CURDATE(), CURTIME(), 'active')",
                        (
                            1, // Default br_id
                            0, // imgID - using 0 as default
                            filename,
                            session_id, // Use session_id as item name
                            "recording", // type
                            id, // user_id
                        )
                    )?;

                    // Get the ID of the inserted recording (last inserted ID)
                    let id: Option<u64> = conn.exec_first("SELECT LAST_INSERT_ID()", ())?;
                    // Update the global flag to indicate database is now available
                    DATABASE_AVAILABLE.store(true, Ordering::SeqCst);
                    Ok(id.unwrap_or(0))
                } else {
                    eprintln!("User with RepID {} not found in salesrep table", user_id);
                    Ok(0) // Return 0 as a placeholder
                }
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

            // Get the salesrep ID (the primary key) from the RepID
            let salesrep_id: Option<u32> = conn.exec_first(
                "SELECT ID FROM salesrep WHERE RepID = ?",
                (user_id,)
            )?;

            if let Some(id) = salesrep_id {
                conn.exec_drop(
                    "INSERT INTO web_images (br_id, imgID, imgName, itmName, type, user_id, date, time, status) VALUES (?, ?, ?, ?, ?, ?, CURDATE(), CURTIME(), 'active')",
                    (
                        1, // Default br_id
                        0, // imgID - using 0 as default
                        filename,
                        session_id, // Use session_id as item name
                        "recording", // type
                        id, // user_id
                    )
                )?;

                // Get the ID of the inserted recording (last inserted ID)
                let id: Option<u64> = conn.exec_first("SELECT LAST_INSERT_ID()", ())?;
                Ok(id.unwrap_or(0))
            } else {
                eprintln!("User with RepID {} not found in salesrep table", user_id);
                Ok(0) // Return 0 as a placeholder
            }
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

        // Ensure user exists in the salesrep table
        create_user(user_id, None, None)?;

        // Get the salesrep ID (the primary key) from the RepID
        let salesrep_id: Option<u32> = conn.exec_first(
            "SELECT ID FROM salesrep WHERE RepID = ?",
            (user_id,)
        )?;

        if let Some(id) = salesrep_id {
            conn.exec_drop(
                "INSERT INTO user_activity (salesrepTb, activity_type, duration, rDateTime) VALUES (?, ?, ?, NOW())",
                (id, activity_type, duration_seconds.unwrap_or(0))
            )?;
        } else {
            eprintln!("User with RepID {} not found in salesrep table", user_id);
        }
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
        // If database is not available, skip saving network usage
        eprintln!("Database not available, skipping network usage save");
        return Ok(());
    }

    // Check if network_usage table exists
    if let Some(ref pool) = *DB_POOL {
        let mut conn = pool.get_conn()?;

        // Skip saving network usage since there's no corresponding table in remote-xwork database
        // The remote-xwork database doesn't have a table for network usage tracking
    } else {
        eprintln!("Database pool is not available");
    }

    // Return Ok to maintain compatibility without actually saving
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

        // Get the salesrep ID (the primary key) from the RepID
        let salesrep_id: Option<u32> = conn.exec_first(
            "SELECT ID FROM salesrep WHERE RepID = ?",
            (user_id,)
        )?;

        if let Some(id) = salesrep_id {
            if let Some(lim) = limit {
                let result = conn.exec_map(
                    "SELECT ID, itmName, imgName, imgName, br_id, date FROM web_images WHERE user_id = ? AND type = 'screenshot' ORDER BY date DESC, time DESC LIMIT ?",
                    (id, lim),
                    |(id, session_id, file_path, filename, file_size, created_at): (u32, String, String, String, i32, String)| {
                        ScreenshotData {
                            id,
                            session_id,
                            file_path,
                            filename,
                            file_size: Some(file_size as i64),
                            created_at,
                        }
                    }
                )?;
                Ok(result)
            } else {
                let result = conn.exec_map(
                    "SELECT ID, itmName, imgName, imgName, br_id, date FROM web_images WHERE user_id = ? AND type = 'screenshot' ORDER BY date DESC, time DESC",
                    (id,),
                    |(id, session_id, file_path, filename, file_size, created_at): (u32, String, String, String, i32, String)| {
                        ScreenshotData {
                            id,
                            session_id,
                            file_path,
                            filename,
                            file_size: Some(file_size as i64),
                            created_at,
                        }
                    }
                )?;
                Ok(result)
            }
        } else {
            eprintln!("User with RepID {} not found in salesrep table", user_id);
            Ok(Vec::new())
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

        // Get the salesrep ID (the primary key) from the RepID
        let salesrep_id: Option<u32> = conn.exec_first(
            "SELECT ID FROM salesrep WHERE RepID = ?",
            (user_id,)
        )?;

        if let Some(id) = salesrep_id {
            if let Some(lim) = limit {
                let result = conn.exec_map(
                    "SELECT ID, itmName, imgName, imgName, br_id, imgID, date FROM web_images WHERE user_id = ? AND type = 'recording' ORDER BY date DESC, time DESC LIMIT ?",
                    (id, lim),
                    |(id, session_id, filename, file_path, br_id, img_id, created_at): (u32, String, String, String, i32, i32, String)| {
                        RecordingData {
                            id,
                            session_id,
                            filename,
                            file_path,
                            duration_seconds: br_id,
                            file_size: img_id as i64,
                            created_at,
                        }
                    }
                )?;
                Ok(result)
            } else {
                let result = conn.exec_map(
                    "SELECT ID, itmName, imgName, imgName, br_id, imgID, date FROM web_images WHERE user_id = ? AND type = 'recording' ORDER BY date DESC, time DESC",
                    (id,),
                    |(id, session_id, filename, file_path, br_id, img_id, created_at): (u32, String, String, String, i32, i32, String)| {
                        RecordingData {
                            id,
                            session_id,
                            filename,
                            file_path,
                            duration_seconds: br_id,
                            file_size: img_id as i64,
                            created_at,
                        }
                    }
                )?;
                Ok(result)
            }
        } else {
            eprintln!("User with RepID {} not found in salesrep table", user_id);
            Ok(Vec::new())
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

        // Get the salesrep ID (the primary key) from the RepID
        let salesrep_id: Option<u32> = conn.exec_first(
            "SELECT ID FROM salesrep WHERE RepID = ?",
            (user_id,)
        )?;

        if let Some(id) = salesrep_id {
            if let Some(lim) = limit {
                let result = conn.exec_map(
                    "SELECT ID, activity_type, duration, rDateTime FROM user_activity WHERE salesrepTb = ? ORDER BY rDateTime DESC LIMIT ?",
                    (id, lim),
                    |(id, activity_type, duration, timestamp): (u32, String, i32, String)| {
                        UserActivityData {
                            id,
                            activity_type,
                            duration_seconds: duration,
                            timestamp,
                        }
                    }
                )?;
                Ok(result)
            } else {
                let result = conn.exec_map(
                    "SELECT ID, activity_type, duration, rDateTime FROM user_activity WHERE salesrepTb = ? ORDER BY rDateTime DESC",
                    (id,),
                    |(id, activity_type, duration, timestamp): (u32, String, i32, String)| {
                        UserActivityData {
                            id,
                            activity_type,
                            duration_seconds: duration,
                            timestamp,
                        }
                    }
                )?;
                Ok(result)
            }
        } else {
            eprintln!("User with RepID {} not found in salesrep table", user_id);
            Ok(Vec::new())
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