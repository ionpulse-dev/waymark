//! Findings: the result of a check. A finding names the broken rule.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub path: String,
    /// A stable rule name. For the conformance fixtures it equals the file name.
    pub rule: &'static str,
    pub severity: Severity,
    pub message: String,
}

impl Finding {
    pub fn error(path: &str, rule: &'static str, message: impl Into<String>) -> Self {
        Finding {
            path: path.to_string(),
            rule,
            severity: Severity::Error,
            message: message.into(),
        }
    }

    pub fn warning(path: &str, rule: &'static str, message: impl Into<String>) -> Self {
        Finding {
            path: path.to_string(),
            rule,
            severity: Severity::Warning,
            message: message.into(),
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

/// True if any finding has severity `Error`.
pub fn has_errors(findings: &[Finding]) -> bool {
    findings.iter().any(Finding::is_error)
}
