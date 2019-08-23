use graphql_parser::Pos;
use std::fmt;

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Debug)]
pub struct Comment {
    pub severity: Severity,
    pub message: String,
}

impl Comment {
    pub fn new(severity: Severity, message: String) -> Self {
        Self { severity, message }
    }
}

#[derive(Clone, Debug)]
pub struct PositionedComment {
    start_pos: Pos,
    end_pos: Pos,
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

#[derive(Debug)]
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
}
