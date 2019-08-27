pub use graphql_parser::Pos;
use serde::Serialize;
use serde_json;
use std::convert::From;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Severity {
    Warning,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Error => write!(f, "error"),
            Self::Warning => write!(f, "warning"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Comment {
    pub severity: Severity,
    pub message: String,
}

impl Comment {
    pub fn new(severity: Severity, message: String) -> Self {
        Self { severity, message }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PositionedComment {
    #[serde(with = "json::PosDef")]
    start_pos: Pos,
    #[serde(with = "json::PosDef")]
    end_pos: Pos,
    #[serde(flatten)]
    comment: Comment,
}

impl PositionedComment {
    pub fn new(start_pos: Pos, comment: Comment) -> Self {
        Self {
            end_pos: start_pos,
            start_pos,
            comment,
        }
    }
}

impl fmt::Display for PositionedComment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] ({} - {})",
            self.start_pos, self.comment.severity, self.comment.message
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CheckResult {
    schema: String,
    comments: Vec<PositionedComment>,
}

impl fmt::Display for CheckResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.comments.is_empty() {
            write!(f, "")
        } else {
            let mut message = String::new();
            for c in &self.comments {
                message.push_str(&format!("{}\n", c));
            }
            write!(f, "{}", message)
        }
    }
}

impl CheckResult {
    pub fn new(schema: String, comments: Vec<PositionedComment>) -> Self {
        Self { schema, comments }
    }

    pub fn to_json(&self) -> crate::Result<String> {
        serde_json::to_string(&self.comments).map_err(|e| e.into())
    }
}

pub enum Format {
    TTY,
    JSON,
}

impl From<&str> for Format {
    fn from(s: &str) -> Self {
        match s {
            "json" => Format::JSON,
            _ => Format::TTY, // fallback to tty
        }
    }
}

impl Default for Format {
    fn default() -> Self {
        Format::TTY
    }
}

mod json {
    use super::*;

    #[derive(Serialize)]
    #[serde(remote = "Pos")]
    pub struct PosDef {
        pub line: usize,
        pub column: usize,
    }
}
