use crate::{
    ascii::{Ascii, DOTS, EIGHT, FIVE, FOUR, NINE, ONE, SEVEN, SIX, THREE, TWO, ZERO},
    error::Error,
    session::{Activity, TimerMessage},
    Result,
};
use serde::{
    de::{self, Visitor},
    Deserialize,
};
use std::{
    fmt::{self, Display},
    str::FromStr,
    sync::mpsc::Sender,
    thread,
    time::{Duration, Instant},
};

/// Pomodoro/Rest timer.
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

    /// Start the [`Timer`].
    pub fn start(&mut self, activity: Activity, tx: &Sender<TimerMessage>) -> Result<()> {
        while self.residue > 0 {
            let time = Instant::now();
            tx.send(TimerMessage::new(
                activity,
                self.to_ascii_art(),
                self.remaining_percentage(),
            ))
            .unwrap(); // TODO: handle thread sending error
            thread::sleep(
                Duration::from_millis(999)
                    .checked_sub(time.elapsed())
                    .ok_or(Error::RenderTime)?,
            );
            self.residue -= 1;
        }

        // Reset `residue`.
        self.residue = self.total;

        Ok(())
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
                    if chars.len() == 0 {
                        return Err(Self::Err::ParseTimer(format!("null {} value", c)));
                    }

                    let v: String = chars.iter().collect(); // Concatenate chars into one string.
                    chars.clear(); // Clear the chars vector for later reuse.

                    // Safe to unwrap since chars are filtered for digits.
                    let mut value = v.parse::<usize>().unwrap();

                    match c {
                        'h' | 'H' => value *= 3600,
                        'm' | 'M' => value *= 60,
                        _ => {},
                    }

                    duration = duration
                        .checked_add(value)
                        .ok_or(Self::Err::TimerOverflow)?;
                }
                c => {
                    if !c.is_digit(10) {
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
                    _ => panic!("valid range for `{}` is 0..9", self),
                },
            );
        }

        ascii_lines.join("\n")
    }
}

struct TimerVisitor {}

impl<'de> Visitor<'de> for TimerVisitor {
    type Value = Timer;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        // TODO: change this error message on deserialization error
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
        deserializer.deserialize_str(TimerVisitor {})
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
