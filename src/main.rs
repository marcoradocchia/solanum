mod args;
mod ascii;
mod config;
mod error;
mod session;
mod timer;
mod ui;

use args::Args;
use config::Config;
use error::Error;
use std::{process::ExitCode, sync::mpsc, thread};

pub type Result<T> = std::result::Result<T, Error>;

fn run() -> Result<()> {
    // Parse CLI arguments.
    let args = Args::new();
    // Parse configuration and override with CLI arguments.
    let config = Config::new(args.get_config_path())?.override_with_args(args);

    // Retrieve `Session` and `Ui` from configuration.
    let (mut session, ui) = config.split();

    // Setup terminal for TUI. 
    let terminal = ui::setup_terminal().map_err(Error::Terminal)?;

    // Channel to send data from logic thread (`session`) to UI thread (`ui`).
    let (tx, rx) = mpsc::channel();

    let renderer = thread::spawn(move || -> Result<()> {
        ui.render(terminal, rx)
    });
    session.start(tx)?;

    renderer.join().unwrap()?;
    // Restore terminal to previous screen and behaviour.
    // ui::restore_terminal(terminal).map_err(Error::Terminal)?;

    Ok(())
}

/// Run program, print message on error and return [`ExitCode`].
fn main() -> ExitCode {
    if let Err(err_msg) = run() {
        eprintln!("error: {err_msg}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
