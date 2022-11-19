use crate::{args::Args, error::Error, session::Session, ui::Ui, Result};
use serde::Deserialize;
use std::{fs, path::Path};

/// Configuration options.
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    /// Ui configuration options.
    #[serde(default)]
    ui: Ui,
    /// Session configuration options.
    #[serde(default)]
    session: Session,
}

impl Config {
    /// Parse configuration from configuration file.
    pub fn new(config_path: Option<&Path>) -> Result<Self> {
        let config_path = match config_path {
            Some(path) => {
                // Argument configuration file path does not exist.
                if !path.is_file() {
                    return Err(Error::ConfigNotFound(path.to_path_buf()));
                }
                path.to_path_buf()
            }
            None => match dirs::config_dir() {
                Some(config_path) => config_path.join("solanum/config.toml"),
                None => return Ok(Self::default()),
            },
        };

        Ok(match fs::read_to_string(config_path) {
            Ok(config) => toml::from_str(&config)?,
            Err(_) => Self::default(),
        })
    }

    /// Override configuration with CLI arguments.
    pub fn override_with_args(mut self, args: Args) -> Self {
        if let Some(pomodoro) = args.get_pomodoro() {
            self.session.pomodoro = pomodoro;
        }

        if let Some(short_break) = args.get_short_break() {
            self.session.short_break = short_break;
        }

        if let Some(long_break) = args.get_long_break() {
            self.session.long_break = long_break;
        }

        if let Some(pomodoros) = args.get_pomodoros() {
            self.session.pomodoros = pomodoros;
        }

        self
    }

    /// Split [`Config`] into tuple for destructuring into [`Session`] and [`Ui`].
    pub fn split(self) -> (Session, Ui) {
        (self.session, self.ui)
    }
}
