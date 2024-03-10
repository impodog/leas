use super::*;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum ErrorData {
    Line(usize),
    Source(Box<dyn std::error::Error>),
}

#[derive(Debug)]
pub struct Error {
    msg: String,
    data: ErrorData,
}

impl Error {
    pub fn new(msg: impl ToString, line: usize) -> Self {
        Self {
            msg: msg.to_string(),
            data: ErrorData::Line(line),
        }
    }

    pub fn with(self, msg: impl ToString) -> Self {
        Self::with_source(self, msg)
    }

    pub fn with_source(err: impl std::error::Error + 'static, msg: impl ToString) -> Self {
        Self {
            msg: msg.to_string(),
            data: ErrorData::Source(Box::new(err)),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data {
            ErrorData::Line(line) => write!(f, "(Line {}) {}", line, self.msg),
            ErrorData::Source(ref err) => write!(f, "[ {} ]\n{}", self.msg, err),
        }
    }
}

impl std::error::Error for Error {}
