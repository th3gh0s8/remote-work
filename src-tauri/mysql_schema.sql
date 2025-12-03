-- MySQL Database Schema for Remote Work Application

-- Create database if it doesn't exist
CREATE DATABASE IF NOT EXISTS remote_work_db;
USE remote_work_db;

-- Table to store screenshot data
CREATE TABLE screenshots (
    id INT AUTO_INCREMENT PRIMARY KEY,
    session_id VARCHAR(255) NOT NULL,
    image_data LONGBLOB NOT NULL,
    filename VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_session_id (session_id),
    INDEX idx_created_at (created_at)
);

-- Table to store recording metadata
CREATE TABLE recordings (
    id INT AUTO_INCREMENT PRIMARY KEY,
    session_id VARCHAR(255) NOT NULL,
    filename VARCHAR(255) NOT NULL,
    file_path VARCHAR(500),
    duration_seconds INT,
    file_size BIGINT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_session_id (session_id),
    INDEX idx_created_at (created_at)
);

-- Table to store recording segments (for concatenated recordings)
CREATE TABLE recording_segments (
    id INT AUTO_INCREMENT PRIMARY KEY,
    recording_id INT,
    segment_number INT NOT NULL,
    filename VARCHAR(255) NOT NULL,
    file_path VARCHAR(500),
    duration_seconds INT,
    file_size BIGINT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (recording_id) REFERENCES recordings(id) ON DELETE CASCADE,
    INDEX idx_recording_id (recording_id)
);

-- Table to store user activity and idle detection data
CREATE TABLE user_activity (
    id INT AUTO_INCREMENT PRIMARY KEY,
    activity_type ENUM('active', 'idle') NOT NULL,
    duration_seconds INT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_timestamp (timestamp)
);

-- Table to store network usage statistics
CREATE TABLE network_usage (
    id INT AUTO_INCREMENT PRIMARY KEY,
    download_speed VARCHAR(50),
    upload_speed VARCHAR(50),
    total_downloaded VARCHAR(50),
    total_uploaded VARCHAR(50),
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_recorded_at (recorded_at)
);

-- Table to store excluded window titles for screenshot masking
CREATE TABLE excluded_windows (
    id INT AUTO_INCREMENT PRIMARY KEY,
    window_title VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Table to store application process status
CREATE TABLE process_status (
    id INT AUTO_INCREMENT PRIMARY KEY,
    recording_active BOOLEAN DEFAULT FALSE,
    screenshotting_active BOOLEAN DEFAULT FALSE,
    idle_detection_active BOOLEAN DEFAULT FALSE,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

-- Insert initial process status
INSERT INTO process_status (recording_active, screenshotting_active, idle_detection_active) 
VALUES (FALSE, FALSE, FALSE);