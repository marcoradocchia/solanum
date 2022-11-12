use std::{fmt, num::ParseIntError, io::Error as IoError};

#[derive(Debug)]
pub enum Error {
    /// Occurs when unable to parse [`Timer`] from stirng.
    ParseTimer(ParseIntError),
    /// Occurs when user tries to setup a [`Timer`] for a number of seconds grater than `usize`.
    TimerOverflow,
    /// Occurs when unable to initalize [`Terminal`] for TUI.
    Terminal(IoError),
    /// Generic error.
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseTimer(err) => write!(f, "{}", err),
            Self::TimerOverflow => write!(f, "exceeded maximum timer duration ({}s)", usize::MAX),
            Self::Terminal(err) => write!(f, "unable to initialize TUI: {}", err),
            Self::Other(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self::Other(err.to_string())
    }
}
