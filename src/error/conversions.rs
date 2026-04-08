use crate::error::structure::AppError;

impl From<String> for AppError {
    fn from(message: String) -> Self {
        Self::unknown(&message)
    }
}

impl From<&str> for AppError {
    fn from(message: &str) -> Self {
        Self::unknown(message)
    }
}
