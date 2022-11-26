mod args;
mod ascii;
mod config;
mod error;
mod event;
mod session;
mod timer;
mod ui;
mod notification;

use args::Args;
use config::Config;
use error::Error;
use event::EventHandler;
use std::{process::ExitCode, sync::mpsc};

pub type Result<T> = std::result::Result<T, Error>;

fn run() -> Result<()> {
    // Parse CLI arguments.
    let args = Args::new();
    // Parse configuration and override with CLI arguments.
    let config = Config::new(args.get_config_path())?.override_with_args(args);

    // Retrieve `Session` and `Ui` from configuration.
    let (mut session, ui) = config.split();

    // Channel to send data from logic thread (`session`) to UI thread (`ui`).
    let (tx_ui, rx_ui) = mpsc::channel();
    // Channel to send events from EventHandler to logic thread (`session`).
    let (tx_event, rx_event) = mpsc::channel();

    // Spawn event handler to handle keyboard events and terminal resize.
    let event_handler_thread = EventHandler::spawn_thread(tx_event, tx_ui.clone());

    // Spawn Ui thread.
    let renderer_thread = ui.spawn_thread(rx_ui)?;
    // Session logic (timers).
    session.start(tx_ui, rx_event)?;

    // Join threads.
    renderer_thread.join().unwrap()?;
    event_handler_thread.join().unwrap()?;

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
