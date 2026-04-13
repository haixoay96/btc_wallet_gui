/// Categorized application error with user-friendly messages
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AppError {
    /// User input validation errors
    Validation { field: String, message: String },
    /// API errors (rate limited, server error, invalid response)
    Api {
        endpoint: String,
        status_code: Option<u16>,
        message: String,
    },
    /// Storage/IO errors
    Storage {
        operation: String,
        path: Option<String>,
        message: String,
    },
    /// Cryptographic errors (wrong passphrase, invalid signature)
    Crypto { operation: String, message: String },
    /// Unknown/unexpected errors
    Unknown { message: String },
}
