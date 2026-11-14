//! The error type shared by all modules. The codes are the error codes of the MCP server
//! (see `specs/foundation/mcp-server`).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    ValidationFailed,
    SyncFailed,
    SetupProblem,
    AccessDenied,
    StaleBase,
    ApprovalDenied,
    ApprovalUnavailable,
    Paused,
    NotFound,
    InvalidArguments,
}

impl Code {
    pub fn as_str(self) -> &'static str {
        match self {
            Code::ValidationFailed => "validation_failed",
            Code::SyncFailed => "sync_failed",
            Code::SetupProblem => "setup_problem",
            Code::AccessDenied => "access_denied",
            Code::StaleBase => "stale_base",
            Code::ApprovalDenied => "approval_denied",
            Code::ApprovalUnavailable => "approval_unavailable",
            Code::Paused => "paused",
            Code::NotFound => "not_found",
            Code::InvalidArguments => "invalid_arguments",
        }
    }
}

/// An error with a code, a message in plain words, and a `next` sentence that says what to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub code: Code,
    pub message: String,
    pub next: Option<String>,
    /// Facts that help the caller fix the call, for example the documents that link to a target.
    pub data: Option<serde_json::Value>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(code: Code, message: impl Into<String>) -> Error {
        Error {
            code,
            message: message.into(),
            next: None,
            data: None,
        }
    }

    pub fn with_next(mut self, next: impl Into<String>) -> Error {
        self.next = Some(next.into());
        self
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Error {
        self.data = Some(data);
        self
    }

    /// A problem that only an update fixes: a newer waymark, or an entry of the MCP config that
    /// this waymark cannot use. The server shows it in `initialize`, in the description of
    /// `status`, and in the error of every tool (see `specs/foundation/mcp-server`).
    pub fn needs_update(message: impl Into<String>, next: impl Into<String>) -> Error {
        Error::new(Code::SetupProblem, message)
            .with_next(next)
            .with_data(serde_json::json!({ "update_needed": true }))
    }

    pub fn is_update_needed(&self) -> bool {
        self.data
            .as_ref()
            .and_then(|d| d.get("update_needed"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn setup(message: impl Into<String>) -> Error {
        Error::new(Code::SetupProblem, message)
    }

    pub fn invalid(message: impl Into<String>) -> Error {
        Error::new(Code::InvalidArguments, message)
    }

    pub fn not_found(message: impl Into<String>) -> Error {
        Error::new(Code::NotFound, message)
    }

    pub fn sync(message: impl Into<String>) -> Error {
        Error::new(Code::SyncFailed, message)
    }

    pub fn validation(message: impl Into<String>) -> Error {
        Error::new(Code::ValidationFailed, message)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for Error {}
