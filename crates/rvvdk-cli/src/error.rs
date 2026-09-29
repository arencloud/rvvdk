use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub(crate) struct Failure {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_error: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
impl Failure {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            os_error: None,
            details: None,
        }
    }
    pub fn io(context: &str, error: std::io::Error) -> Self {
        Self {
            code: "io",
            message: format!("{context}: {error}"),
            os_error: error.raw_os_error(),
            details: None,
        }
    }
}
impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}
impl From<rvvdk_core::Error> for Failure {
    fn from(error: rvvdk_core::Error) -> Self {
        let cancelled = error.is_cancelled();
        if let Some(failure) = error.copy_failure() {
            let mut result = Self::new(
                if cancelled {
                    "cancelled"
                } else {
                    "copy_failed"
                },
                error.to_string(),
            );
            result.details = Some(serde_json::json!({
                "backend": failure.backend, "operation": format!("{:?}", failure.operation),
                "offset": failure.range.map(|r| r.offset()), "length": failure.range.map(|r| r.length()),
                "confirmed_bytes_read": failure.progress.bytes_read,
                "confirmed_bytes_written": failure.progress.bytes_written,
                "confirmed_bytes_zeroed": failure.progress.bytes_zeroed,
                "confirmed_bytes_discarded": failure.progress.bytes_discarded,
                "unconfirmed_io": failure.progress.unconfirmed_io
            }));
            return result;
        }
        let code = match &error {
            rvvdk_core::Error::Cancelled => "cancelled",
            rvvdk_core::Error::VerificationMismatch { offset } => {
                let mut result = Self::new("verification_mismatch", error.to_string());
                result.details = Some(serde_json::json!({"mismatch_offset": offset}));
                return result;
            }
            rvvdk_core::Error::MemoryBudgetExceeded { .. }
            | rvvdk_core::Error::MemoryAccountingOverflow => "memory_budget",
            rvvdk_core::Error::ConcurrentFileAccess { .. } => "concurrent_access",
            rvvdk_core::Error::Io(e) => {
                return Self {
                    code: "io",
                    message: e.to_string(),
                    os_error: e.raw_os_error(),
                    details: None,
                };
            }
            _ => "planning",
        };
        Self::new(code, error.to_string())
    }
}
