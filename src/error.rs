use std::{error, fmt, io, num, path::PathBuf};

/// Runtime handled errors.
#[derive(Debug)]
pub enum Error {
    /// Occurs when unable to parse [`Timer`](crate::timer::Timer) from string.
    ParseTimer(String),
    /// Occurs when user tries to setup a [`Timer`](crate::timer::Timer) for a number of seconds grater
    /// than [`usize`](usize::MAX).
    TimerOverflow,
    /// Occurs when unable to initalize [`Terminal`](tui::Terminal) for TUI.
    Terminal(io::Error),
    /// Occurs when TUI rendering takes more than 1 second, making the timer unreliable.
    RenderTime,
    /// Occurs when given configuration file path does not exist.
    ConfigNotFound(PathBuf),
    /// Occurs on broken configuration.
    Config(toml::de::Error),
    /// Generic error.
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseTimer(err) => write!(f, "unable to parse duration: {}", err),
            Self::TimerOverflow => write!(f, "exceeded maximum timer duration ({}s)", usize::MAX),
            Self::Terminal(err) => write!(f, "unable to initialize TUI: {}", err),
            Self::RenderTime => write!(f, "TUI rendering takes too long"),
            Self::ConfigNotFound(path) => {
                write!(f, "configuration file not found at specified location: '{}'", path.display())
            }
            Self::Config(err) => write!(f, "broken configuration: {}", err),
            Self::Other(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for Error {}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Self::Config(err)
    }
}
