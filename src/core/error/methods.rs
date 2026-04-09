use crate::core::error::structure::AppError;
use crate::ui::i18n::t;

impl AppError {
    /// Convenience constructors
    pub fn validation(field: &str, message: &str) -> Self {
        Self::Validation {
            field: field.to_string(),
            message: message.to_string(),
        }
    }
    pub fn api_with_status(endpoint: &str, status: u16, message: &str) -> Self {
        Self::Api {
            endpoint: endpoint.to_string(),
            status_code: Some(status),
            message: message.to_string(),
        }
    }
    pub fn storage(operation: &str, message: &str) -> Self {
        Self::Storage {
            operation: operation.to_string(),
            path: None,
            message: message.to_string(),
        }
    }
    pub fn storage_with_path(operation: &str, path: &str, message: &str) -> Self {
        Self::Storage {
            operation: operation.to_string(),
            path: Some(path.to_string()),
            message: message.to_string(),
        }
    }
    pub fn crypto(operation: &str, message: &str) -> Self {
        Self::Crypto {
            operation: operation.to_string(),
            message: message.to_string(),
        }
    }
    pub fn unknown(message: &str) -> Self {
        Self::Unknown {
            message: message.to_string(),
        }
    }

    /// Get user-facing error message (already localized via t!())
    pub fn user_message(&self) -> String {
        match self {
            Self::Validation { message, .. }
            | Self::Api { message, .. }
            | Self::Storage { message, .. }
            | Self::Crypto { message, .. }
            | Self::Unknown { message } => message.clone(),
        }
    }

    /// Get detailed context for debugging
    pub fn context(&self) -> Option<String> {
        match self {
            Self::Validation { field, .. } => Some(format!("Field: {field}")),
            Self::Api {
                endpoint,
                status_code,
                ..
            } => {
                let status = status_code.map(|s| format!("HTTP {s}")).unwrap_or_default();
                Some(format!("Endpoint: {endpoint} {status}").trim().to_string())
            }
            Self::Storage {
                operation, path, ..
            } => {
                let p = path.as_deref().unwrap_or("");
                Some(format!("Op: {operation} Path: {p}").trim().to_string())
            }
            Self::Crypto { operation, .. } => Some(format!("Op: {operation}")),
            Self::Unknown { .. } => None,
        }
    }

    /// Get a short title for the error
    pub fn title(&self) -> String {
        match self {
            Self::Validation { .. } => t("Lỗi nhập liệu", "Validation Error").to_string(),
            Self::Api { .. } => t("Lỗi API", "API Error").to_string(),
            Self::Storage { .. } => t("Lỗi lưu trữ", "Storage Error").to_string(),
            Self::Crypto { .. } => t("Lỗi mã hóa", "Cryptographic Error").to_string(),
            Self::Unknown { .. } => t("Lỗi không xác định", "Unknown Error").to_string(),
        }
    }

    /// Get a suggested action for the user
    pub fn suggestion(&self) -> String {
        match self {
            Self::Validation { field, .. } => {
                format!("{}: {}", t("Kiểm tra lại", "Please check"), field)
            }
            Self::Api { endpoint, .. } => {
                format!(
                    "{}: {endpoint}",
                    t("Lỗi kết nối API", "API connection failed")
                )
            }
            Self::Storage {
                operation, path, ..
            } => match (operation.as_str(), path) {
                ("export", _) => t("Kiểm tra dung lượng ổ đĩa", "Check disk space").to_string(),
                (_, Some(p)) => format!("{}: {p}", t("Lỗi tại đường dẫn", "Path error")),
                _ => t("Kiểm tra quyền truy cập", "Check permissions").to_string(),
            },
            Self::Crypto { operation, .. } => match operation.as_str() {
                "login" => t("Kiểm tra lại passphrase", "Check your passphrase").to_string(),
                _ => t(
                    "Kiểm tra lại passphrase hoặc backup",
                    "Check your passphrase or backup",
                )
                .to_string(),
            },
            Self::Unknown { .. } => t(
                "Thử lại hoặc khởi động lại ứng dụng",
                "Try again or restart the application",
            )
            .to_string(),
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Api { .. } | Self::Storage { .. })
    }
}
