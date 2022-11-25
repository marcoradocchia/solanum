use crate::{
    ascii::{Ascii, DOTS, EIGHT, FIVE, FOUR, NINE, ONE, SEVEN, SIX, THREE, TWO, ZERO},
    error::Error,
    event::Event,
    notification::notify,
    session::Activity,
    ui::UiCommand,
    Result,
};
use serde::{
    de::{self, Visitor},
    Deserialize,
};
use std::{
    fmt::{self, Display},
    str::FromStr,
    sync::mpsc::{Receiver, RecvError, RecvTimeoutError, Sender},
    time::{Duration, Instant},
};

/// Duration of the _timer expired_ screen in seconds.
const EXPIRED_DURATION: u64 = 5;

/// [`Timer`] status.
#[derive(Debug, Clone)]
pub enum TimerStatus {
    Running(TimerData),
    Paused,
    Expired,
}

/// [`Timer`] data for [`Ui`](crate::ui::Ui) rendering.
///
/// # Note
/// This struct will be passed from [`Timer`] thread to [`Ui`](crate::ui::Ui) thread: for this
/// reason, in `new` method `perc` is casted to `u16` to reduce memory usage.
#[derive(Debug, Clone)]
pub struct TimerData {
    /// Current [`Activity`].
    pub activity: Activity,
    /// ASCII art timer.
    pub ascii: String,
    /// Timer remaining percentage.
    pub perc: u16,
}

impl TimerData {
    pub fn new(activity: Activity, ascii: String, perc: f32) -> Self {
        Self {
            activity,
            ascii,
            perc: perc as u16,
        }
    }
}

impl Default for TimerData {
    fn default() -> Self {
        Self {
            activity: Activity::Pomodoro(0),
            ascii: String::default(),
            perc: 100,
        }
    }
}

/// Pomodoro/Break timer.
#[derive(Debug, Clone, Copy, Default)]
pub struct Timer {
    /// Total duration of the [`Timer`].
    total: usize,
    /// Residue duration of the [`Timer`].
    residue: usize,
}

impl Timer {
    /// Construct a new [`Timer`].
    pub fn new(hours: usize, minutes: usize, seconds: usize) -> Self {
        // It's ok to panic for overflow since this method is not exposed to CLI user
        // (see `FromStr` implementation instad).
        let duration = seconds + (minutes + hours * 60) * 60;
        Self {
            total: duration,
            residue: duration,
        }
    }

    /// Convert [`Timer`] inner value (seconds) to hours/minutes/seconds.
    fn hms(&self) -> (usize, usize, usize) {
        let mut seconds = self.residue;
        let hours = seconds / 3600;
        seconds %= 3600;
        let minutes = seconds / 60;
        seconds %= 60;

        (hours, minutes, seconds)
    }

    /// Convert [`Timer`] to `String` as "HH:MM:SS".
    fn hhmmss(&self) -> String {
        let (hours, minutes, seconds) = self.hms();
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    /// Return the remaining percentage of the [`Timer`].
    fn remaining_percentage(&self) -> f32 {
        (self.residue as f32 / self.total as f32) * 100.0
    }

    /// Start [`Timer`].
    ///
    /// Return value of `true` indicates to the caller that application must be closed.
    pub fn start(
        &mut self,
        activity: Activity,
        tx_ui: &Sender<UiCommand>,
        rx_event: &Receiver<Event>,
    ) -> Result<bool> {
        let delay = |time: Instant| -> Result<Duration> {
            Duration::from_millis(999)
                .checked_sub(time.elapsed())
                .ok_or(Error::RenderTime)
        };

        // Countdown loop.
        while self.residue > 0 {
            let start = Instant::now();
            tx_ui
                .send(UiCommand::Draw(TimerStatus::Running(TimerData::new(
                    activity,
                    self.to_ascii_art(),
                    self.remaining_percentage(),
                ))))
                .unwrap();

            // Let 1 second pass while still being responsive to events.
            // Receiving `RecvTimeoutError::Timeout` means the delay reached Timeout with no
            // events.
            match rx_event.recv_timeout(delay(start)?) {
                Err(RecvTimeoutError::Timeout) => {}
                Ok(Event::TogglePause) => {
                    // Send Pause screen to Ui and wait until next event.
                    tx_ui.send(UiCommand::Draw(TimerStatus::Paused)).unwrap();
                    match rx_event.recv() {
                        Ok(Event::TogglePause) => continue,
                        Ok(Event::Skip) => break,
                        Err(RecvError) => return Ok(true),
                    }
                }
                Ok(Event::Skip) => break,
                // EventHandler disconnected, cose application.
                Err(RecvTimeoutError::Disconnected) => return Ok(true),
            };

            self.residue -= 1;
        }

        // Send desktop notification.
        notify(activity)?;
        // Send Expired screen to Ui, meanwhile listen for events.
        tx_ui.send(UiCommand::Draw(TimerStatus::Expired)).unwrap();
        let start = Instant::now();
        while start.elapsed() <= Duration::from_secs(EXPIRED_DURATION) {
            match rx_event.recv_timeout(Duration::from_secs(EXPIRED_DURATION)) {
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return Ok(true),
                Ok(_) => {}
            }
        }

        // Reset `residue`.
        self.residue = self.total;

        Ok(false)
    }
}

impl Display for Timer {
    /// Default formatter.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (hours, minutes, seconds) = self.hms();

        if hours > 0 {
            write!(f, "{}h", hours)?;
        }
        if minutes > 0 {
            write!(f, "{}m", minutes)?;
        }
        if seconds > 0 {
            write!(f, "{}s", seconds)?;
        }

        Ok(())
    }
}

impl FromStr for Timer {
    type Err = crate::error::Error;
    /// Parse [`Timer`] from string (e.g. "1h2m30s").
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if !s.ends_with(['h', 'H', 'm', 'M', 's', 'S']) {
            return Err(Self::Err::ParseTimer(format!(
                "expected `[[<H>h]<M>m]<S>s` format (found `{}`)",
                s
            )));
        }

        let mut chars: Vec<char> = vec![];
        let mut duration: usize = 0;

        for c in s.chars() {
            match c {
                'h' | 'H' | 'm' | 'M' | 's' | 'S' => {
                    if chars.is_empty() {
                        return Err(Self::Err::ParseTimer(format!("null {} value", c)));
                    }

                    let v: String = chars.iter().collect(); // Concatenate chars into one string.
                    chars.clear(); // Clear the chars vector for later reuse.

                    // Safe to unwrap since chars are filtered for digits.
                    let mut value = v.parse::<usize>().unwrap();

                    match c {
                        'h' | 'H' => value *= 3600,
                        'm' | 'M' => value *= 60,
                        _ => {}
                    }

                    duration = duration
                        .checked_add(value)
                        .ok_or(Self::Err::TimerOverflow)?;
                }
                c => {
                    if !c.is_ascii_digit() {
                        return Err(Self::Err::ParseTimer(format!("`{}` is not a digit", c)));
                    }
                    chars.push(c);
                }
            }
        }

        Ok(Self {
            total: duration,
            residue: duration,
        })
    }
}

impl Ascii for Timer {
    // Convert [`Timer`] to ASCII art.
    fn to_ascii_art(&self) -> String {
        let mut ascii_lines: [String; 5] = Default::default();
        let push_ascii = |ascii_art: &mut [String; 5], lines: [&str; 5]| {
            for i in 0..5 {
                ascii_art[i].push_str(lines[i]);
            }
        };

        for digit in self.hhmmss().chars() {
            push_ascii(
                &mut ascii_lines,
                match digit {
                    ':' => DOTS,
                    '1' => ONE,
                    '2' => TWO,
                    '3' => THREE,
                    '4' => FOUR,
                    '5' => FIVE,
                    '6' => SIX,
                    '7' => SEVEN,
                    '8' => EIGHT,
                    '9' => NINE,
                    '0' => ZERO,
                    _ => unreachable!(),
                },
            );
        }

        ascii_lines.join("\n")
    }
}

struct TimerVisitor;

impl<'de> Visitor<'de> for TimerVisitor {
    type Value = Timer;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("`[[<H>h]<M>m]<S>s` format")
    }

    fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Timer::from_str(v).map_err(de::Error::custom)
    }
}

impl<'de> Deserialize<'de> for Timer {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(TimerVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
