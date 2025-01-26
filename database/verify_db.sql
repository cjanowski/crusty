-- Create database if not exists
CREATE DATABASE IF NOT EXISTS crusty;
USE crusty;

-- Create users table if not exists
CREATE TABLE IF NOT EXISTS users (
    id BINARY(16) NOT NULL,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    INDEX idx_email (email)
);

-- Create UUID helper functions
DELIMITER //

CREATE FUNCTION IF NOT EXISTS BIN_TO_UUID(b BINARY(16))
RETURNS VARCHAR(36)
DETERMINISTIC
BEGIN
   RETURN LOWER(CONCAT_WS('-',
        HEX(SUBSTR(b,  5, 4)),
        HEX(SUBSTR(b,  3, 2)),
        HEX(SUBSTR(b,  1, 2)),
        HEX(SUBSTR(b,  9, 2)),
        HEX(SUBSTR(b, 11, 6))
    ));
END //

CREATE FUNCTION IF NOT EXISTS UUID_TO_BIN(s VARCHAR(36))
RETURNS BINARY(16)
DETERMINISTIC
BEGIN
    RETURN UNHEX(CONCAT(
        SUBSTR(s, 15, 4),
        SUBSTR(s, 10, 4),
        SUBSTR(s,  1, 8),
        SUBSTR(s, 20, 4),
        SUBSTR(s, 25)
    ));
END //

DELIMITER ;

-- Drop existing media table if it exists
DROP TABLE IF EXISTS media;

-- Create media table
CREATE TABLE media (
    id BINARY(16) NOT NULL,
    user_id BINARY(16) NOT NULL,
    filename VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(255) NOT NULL,
    encrypted_key VARBINARY(512) NOT NULL,
    encryption_iv VARBINARY(16) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_id (user_id)
); 