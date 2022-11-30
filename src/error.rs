use std::{env, error, fmt, io, path::PathBuf, sync::mpsc};

use crate::figlet;

/// Runtime handled errors.
#[derive(Debug)]
pub enum Error {
    /// Occurs when unable to parse [`Timer`](crate::timer::Timer) from string.
    ParseTimer(String),
    /// Occurs when user tries to setup a [`Timer`](crate::timer::Timer) for a number of seconds
    /// grater than [`usize`](usize::MAX).
    TimerOverflow,
    /// Occurs when unable to initalize [`Terminal`](tui::Terminal) for TUI.
    Terminal(io::Error),
    /// Occurs when TUI rendering takes more than 1 second, making the timer unreliable.
    RenderTime,
    /// Occurs when EventHandler hangs up, making the application unresponsive.
    EventHandlerHangUp,
    /// Occurs when given configuration file path does not exist.
    ConfigNotFound(PathBuf),
    /// Occurs on broken configuration.
    Config(toml::de::Error),
    /// Occurs when unable to send notifications.
    Notification(notify_rust::error::Error),
    /// Occurs when `HOME` environment variable is not set while expanding `~` in path.
    HomeNotFound,
    /// Occurs when environment variable found in path is not set.
    EnvVar(String, env::VarError),
    /// Occurs when trying to convert `Path` to non-UTF8 string.
    NonUtf8Path(PathBuf),
    /// Occurs provided `.flf` is not a proper FIGlet font file.
    Font(figlet::FontError),
    /// Generic error.
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseTimer(err) => write!(f, "unable to parse duration: {}", err),
            Self::TimerOverflow => write!(f, "exceeded maximum timer duration ({}s)", usize::MAX),
            Self::Terminal(err) => write!(f, "terminal error: {}", err),
            Self::RenderTime => write!(f, "TUI rendering takes too long"),
            Self::EventHandlerHangUp => write!(f, "event handler has hang up unexpectedly"),
            Self::ConfigNotFound(path) => {
                write!(f, "configuration file not found at `{}`", path.display())
            }
            Self::Config(err) => write!(f, "broken configuration: {}", err),
            Self::Notification(err) => write!(f, "issue on sending desktop notification: {}", err),
            Self::HomeNotFound => write!(
                f,
                "unable to expand `~`: `HOME` environment variable not set"
            ),
            Self::EnvVar(var, err) => write!(
                f,
                "unable to access environment variable `{}`: {}",
                var, err
            ),
            Self::NonUtf8Path(path) => {
                write!(f, "`{}` contains non-UTF8 characters", path.display())
            }
            Self::Font(err) => write!(f, "invalid FIGlet font file: {}", err),
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

impl From<crossterm::ErrorKind> for Error {
    fn from(err: crossterm::ErrorKind) -> Self {
        Self::Terminal(err)
    }
}

impl From<mpsc::RecvError> for Error {
    fn from(_: mpsc::RecvError) -> Self {
        Self::EventHandlerHangUp
    }
}

impl From<notify_rust::error::Error> for Error {
    fn from(err: notify_rust::error::Error) -> Self {
        Self::Notification(err)
    }
}

impl From<figlet::FontError> for Error {
    fn from(err: figlet::FontError) -> Self {
        Self::Font(err)
    }
}
