use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub(crate) struct Failure {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_error: Option<i32>,
}
impl Failure {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            os_error: None,
        }
    }
    pub fn io(context: &str, error: std::io::Error) -> Self {
        Self {
            code: "io",
            message: format!("{context}: {error}"),
            os_error: error.raw_os_error(),
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
        let code = match &error {
            rvvdk_core::Error::MemoryBudgetExceeded { .. }
            | rvvdk_core::Error::MemoryAccountingOverflow => "memory_budget",
            rvvdk_core::Error::ConcurrentFileAccess { .. } => "concurrent_access",
            rvvdk_core::Error::Io(e) => {
                return Self {
                    code: "io",
                    message: e.to_string(),
                    os_error: e.raw_os_error(),
                };
            }
            _ => "planning",
        };
        Self::new(code, error.to_string())
    }
}
