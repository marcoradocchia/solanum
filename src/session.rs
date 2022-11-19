use crate::{timer::Timer, Result};
use serde::Deserialize;
use std::{
    fmt::{self, Display},
    sync::mpsc::Sender,
};

/// Kind of activity associated to the timer.
#[derive(Debug, Clone, Copy)]
pub enum Activity {
    Pomodoro(u8),
    ShortBreak,
    LongBreak,
}

impl Display for Activity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Pomodoro(num) => write!(f, "Pomodoro #{}", num),
            Self::ShortBreak => write!(f, "Short break"),
            Self::LongBreak => write!(f, "Long break"),
        }
    }
}

/// Thread messages.
pub struct TimerMessage {
    /// Current [`Activity`].
    pub activity: Activity,
    /// ASCII art timer.
    pub ascii: String,
    /// Timer remaining percentage.
    pub perc: f32,
}

impl TimerMessage {
    pub fn new(activity: Activity, ascii: String, perc: f32) -> Self {
        Self {
            activity,
            ascii,
            perc,
        }
    }
}

/// **Solanum** session.
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Session {
    /// Count of completed pomorodos.
    #[serde(skip)]
    pub pomodoro_count: u8,
    /// Pomodoro duration.
    #[serde(default = "default_pomodoro")]
    pub pomodoro: Timer,
    /// Short break duration.
    #[serde(default = "default_short_break")]
    pub short_break: Timer,
    /// Long break duration.
    #[serde(default = "default_long_break")]
    pub long_break: Timer,
    /// Pomodoros before long break.
    #[serde(default = "default_pomodoros")]
    pub pomodoros: u8,
}

#[inline]
fn default_pomodoro() -> Timer {
    Timer::new(0, 25, 0)
}

#[inline]
fn default_short_break() -> Timer {
    Timer::new(0, 5, 0)
}

#[inline]
fn default_long_break() -> Timer {
    Timer::new(0, 15, 0)
}

#[inline]
fn default_pomodoros() -> u8 {
    4
}

impl Default for Session {
    fn default() -> Self {
        Self {
            pomodoro_count: 0,
            pomodoro: default_pomodoro(),
            short_break: default_short_break(),
            long_break: default_long_break(),
            pomodoros: default_pomodoros(),
        }
    }
}

impl Session {
    /// Start [`Session`].
    pub fn start(&mut self, tx: Sender<TimerMessage>) -> Result<()> {
        loop {
            loop {
                // Increase counter and start pomodoro.
                self.pomodoro_count += 1;
                self.pomodoro.start(Activity::Pomodoro(self.pomodoro_count), &tx)?;

                // Jump to long break every <self.pomodoros> completed pomodoros.
                if self.pomodoro_count % self.pomodoros == 0 {
                    break;
                }

                // Start short break.
                self.short_break.start(Activity::ShortBreak, &tx)?;
            }
            // Start long break.
            self.long_break.start(Activity::LongBreak, &tx)?;
        }
    }
}
