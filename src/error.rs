use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    Io(std::io::Error),
    Db(rusqlite::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
    Hci(String),
    Camera(String),
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(e) => write!(f, "IO Error: {}", e),
            AppError::Db(e) => write!(f, "Database Error: {}", e),
            AppError::Http(e) => write!(f, "HTTP Error: {}", e),
            AppError::Json(e) => write!(f, "JSON Error: {}", e),
            AppError::Hci(msg) => write!(f, "HCI Bluetooth Error: {}", msg),
            AppError::Camera(msg) => write!(f, "Camera Error: {}", msg),
            AppError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Db(err)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::Http(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Json(err)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
