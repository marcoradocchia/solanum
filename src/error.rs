use std::{error, fmt, io, path::PathBuf, sync::mpsc};

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
                write!(
                    f,
                    "configuration file not found at specified location: '{}'",
                    path.display()
                )
            }
            Self::Config(err) => write!(f, "broken configuration: {}", err),
            Self::Notification(err) => write!(f, "issue on sending desktop notification: {}", err),
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
