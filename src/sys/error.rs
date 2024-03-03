use super::*;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    msg: String,
    line: usize,
    source: Option<Box<dyn std::error::Error>>,
}

impl Error {
    pub fn new(msg: impl ToString, line: usize) -> Self {
        Self {
            msg: msg.to_string(),
            line,
            source: None,
        }
    }

    pub fn with(self, msg: impl ToString, line: usize) -> Self {
        Self::with_source(self, msg, line)
    }

    pub fn with_source(
        err: impl std::error::Error + 'static,
        msg: impl ToString,
        line: usize,
    ) -> Self {
        Self {
            msg: msg.to_string(),
            line,
            source: Some(Box::new(err)),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.source.as_ref() {
            None => {
                write!(f, "Line {}: {}.", self.line, self.msg)
            }
            Some(source) => {
                write!(
                    f,
                    "[ Line {}: {} ]\n{}",
                    self.line,
                    self.msg,
                    source.as_ref()
                )
            }
        }
    }
}

impl std::error::Error for Error {}
