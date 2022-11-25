use crate::{event::Event, timer::Timer, ui::UiCommand, Result};
use serde::Deserialize;
use std::{
    fmt::{self, Display},
    sync::mpsc::{Receiver, Sender},
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
    pub fn start(&mut self, tx_ui: Sender<UiCommand>, rx_event: Receiver<Event>) -> Result<()> {
        loop {
            loop {
                // Increase counter and start pomodoro.
                self.pomodoro_count += 1;
                if self.pomodoro.start(
                    Activity::Pomodoro(self.pomodoro_count),
                    &tx_ui,
                    &rx_event,
                )? {
                    return Ok(());
                };

                // Jump to long break every <self.pomodoros> completed pomodoros.
                if self.pomodoro_count % self.pomodoros == 0 {
                    break;
                }

                // Start short break.
                if self
                    .short_break
                    .start(Activity::ShortBreak, &tx_ui, &rx_event)?
                {
                    return Ok(());
                };
            }
            // Start long break.
            if self
                .long_break
                .start(Activity::LongBreak, &tx_ui, &rx_event)?
            {
                return Ok(());
            };
        }
    }
}
