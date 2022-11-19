use crate::timer::Timer;
use clap::Parser;
use std::path::{Path, PathBuf};

/// CLI arguments.
#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Pomodoro duration.
    #[arg(short, long)]
    pomodoro: Option<Timer>,
    /// Short break duration.
    #[arg(short, long)]
    short_break: Option<Timer>,
    /// Long rest duration.
    #[arg(short, long)]
    long_break: Option<Timer>,
    /// Pomodoros before long break.
    #[arg(short = 'n', long)]
    pomodoros: Option<u8>,
    /// Custom configuration path.
    #[arg(short, long)]
    config: Option<PathBuf>,
}

impl Args {
    /// Parse CLI arguments.
    pub fn new() -> Self {
        Self::parse()
    }

    /// Getter method for `config` filed.
    #[inline]
    pub fn get_config_path(&self) -> Option<&Path> {
        self.config.as_deref()
    }

    /// Getter method for `pomodoro` filed.
    #[inline]
    pub fn get_pomodoro(&self) -> Option<Timer> {
        self.pomodoro
    }

    /// Getter method for `short_break` filed.
    #[inline]
    pub fn get_short_break(&self) -> Option<Timer> {
        self.short_break
    }

    /// Getter method for `long_break` filed.
    #[inline]
    pub fn get_long_break(&self) -> Option<Timer> {
        self.long_break
    }

    /// Getter method for `pomodoros` filed.
    #[inline]
    pub fn get_pomodoros(&self) -> Option<u8> {
        self.pomodoros
    }
}
