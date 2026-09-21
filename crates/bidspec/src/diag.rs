use std::fmt;

/// An error in a `.bid` file, with its location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: String,
    /// 1-based line.
    pub line: usize,
    /// 1-based column (characters), 0 when the whole line is meant.
    pub col: usize,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.col > 0 {
            write!(
                f,
                "{}:{}:{}: {}",
                self.file, self.line, self.col, self.message
            )
        } else {
            write!(f, "{}:{}: {}", self.file, self.line, self.message)
        }
    }
}

impl std::error::Error for Diagnostic {}
