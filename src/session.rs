use clap::Parser;
use std::io;
use tui::{backend::CrosstermBackend, Terminal};
use crate::timer::Timer;

/// Intervals.
enum IntervalKind {
    Pomodoro,
    Rest,
}

/// `solanum` session.
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Session {
    /// Count of completed pomorodos.
    #[arg(skip)]
    pomodoro_count: u8,
    /// Count of the completed breaks.
    #[arg(skip)]
    rest_count: u8,

    /// Pomodoro duration.
    #[arg(short, long, default_value_t = Timer::new(0, 25, 0))]
    pomodoro: Timer,
    /// Rest duration.
    #[arg(short, long, default_value_t = Timer::new(0, 5, 0))]
    rest: Timer,
    /// Long rest duration.
    #[arg(short, long, default_value_t = Timer::new(0, 15, 0))]
    long_rest: Timer,
    /// Pomodoros before long break.
    #[arg(short = 'n', long, default_value_t = 4)]
    pomodoros: u8,
}

impl Session {
    /// Parse [`Session`] CLI arguments.
    pub fn cli() -> Self {
        Self::parse()
    }

    /// Run timer associated to the given [`IntervalKind`].
    fn timer(&mut self, kind: IntervalKind) {
        match kind {
            IntervalKind::Pomodoro => self.pomodoro.start(),
            IntervalKind::Rest => self.rest.start(),
        }
    }

    /// Start [`Session`].
    pub fn start(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) {
        loop {
            while self.pomodoro_count % self.pomodoros != 0 {
                self.timer(IntervalKind::Pomodoro);
                self.pomodoro_count += 1;
            }

            self.timer(IntervalKind::Rest);
            self.rest_count += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::session::Timer;
    use std::str::FromStr;

    #[test]
    /// Timer displaying as `_h_m_s` format.
    fn timer_display() {
        assert_eq!(Timer::new(2, 10, 4).to_string(), "2h10m4s");
        assert_eq!(Timer::new(0, 2, 122).to_string(), "4m2s");
        assert_eq!(Timer::new(1, 0, 10).to_string(), "1h10s");
        assert_eq!(Timer::new(0, 5, 0).to_string(), "5m");
    }

    #[test]
    /// Timer constructing from string.
    fn timer_from_string() {
        assert_eq!(Timer::from_str("2h10m4s").unwrap().to_string(), "2h10m4s");
        assert_eq!(Timer::from_str("4m2s").unwrap().to_string(), "4m2s");
        assert_eq!(Timer::from_str("1h10s").unwrap().to_string(), "1h10s");
        assert_eq!(Timer::from_str("5m").unwrap().to_string(), "5m");
    }
}
