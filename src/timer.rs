use crate::ascii::{Ascii, DOTS, EIGHT, FIVE, FOUR, NINE, ONE, SEVEN, SIX, THREE, TWO, ZERO};
use std::{fmt::Display, str::FromStr, thread, time::Duration};

/// TUI screens for timer.
enum Screen {
    Running,
    Pause,
    Expired,
}

/// Pomodoro/Rest timer.
#[derive(Debug, Clone, Default)]
pub struct Timer(usize);

impl Timer {
    /// Construct a new [`Timer`].
    pub fn new(hours: usize, minutes: usize, seconds: usize) -> Self {
        // It's ok to panic for overflow since this method is not exposed to CLI user 
        // (see `FromStr` implementation instad).
        Self(seconds + (minutes + hours * 60) * 60)
    }

    /// Convert [`Timer`] inner value (seconds) to hours/minutes/seconds.
    fn to_hms(&self) -> (usize, usize, usize) {
        let mut seconds = self.0;
        let hours = seconds / 3600;
        seconds %= 3600;
        let minutes = seconds / 60;
        seconds %= 60;

        (hours, minutes, seconds)
    }

    /// Convert [`Timer`] to `String` as "HH:MM:SS".
    fn to_hhmmss(&self) -> String {
        let (hours, minutes, seconds) = self.to_hms();
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    /// Render the [`Timer`] screen.
    fn render(&self, screen: Screen) {
        match screen {
            Screen::Running => todo!(),
            Screen::Pause => todo!(),
            Screen::Expired => todo!(),
        }
    }

    /// Start the [`Timer`].
    pub fn start(&mut self) {
        while self.0 > 0 {
            self.render(Screen::Running);
            self.0 -= 0;
            thread::sleep(Duration::from_millis(999));
        }

        self.render(Screen::Expired);
    }
}

impl Display for Timer {
    /// Default formatter.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (hours, minutes, seconds) = self.to_hms();

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
    /// Parse Timer from `&str` (e.g. "1h2m30s").
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars: Vec<char> = vec![];
        let mut timer = Self::default();

        let increment_timer =
            |chars: &mut Vec<char>, factor: usize, timer: &mut Timer| -> Result<(), Self::Err> {
                let v: String = chars.iter().collect(); // Concatenate chars into one string.
                (*chars).clear(); // Clear the chars vector for later reuse.
                timer.0 = timer
                    .0
                    .checked_add(v.parse::<usize>().map_err(Self::Err::ParseTimer)? * factor)
                    .ok_or(Self::Err::TimerOverflow)?;
                Ok(())
            };

        for c in s.chars() {
            match c {
                'h' | 'H' => increment_timer(&mut chars, 3600, &mut timer)?,
                'm' | 'M' => increment_timer(&mut chars, 60, &mut timer)?,
                's' | 'S' => increment_timer(&mut chars, 1, &mut timer)?,
                c => chars.push(c),
            }
        }

        Ok(timer)
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

        for digit in self.to_hhmmss().chars() {
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
