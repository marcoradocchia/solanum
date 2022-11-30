mod args;
mod config;
mod error;
mod event;
mod figlet;
mod notification;
mod path;
mod session;
mod timer;
mod ui;

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
    let config = Config::new(args.get_config_path())?.override_with_args(args)?;

    // Retrieve `Session` and `Ui` from configuration.
    let (mut session, ui) = config.split();

    // Channel to send data from logic thread (`session`) to UI thread (`ui`).
    let (tx_ui, rx_ui) = mpsc::channel();
    // Channel to send events from EventHandler to logic thread (`session`).
    let (tx_event, rx_event) = mpsc::channel();
    // Channel to send termination to event handler.
    let (tx_termination, rx_termination) = mpsc::channel();

    // Spawn event handler to handle keyboard events and terminal resize.
    let event_handler_thread = EventHandler::spawn_thread(tx_event, tx_ui.clone(), rx_termination);

    // Spawn Ui thread.
    let renderer_thread = ui.spawn_thread(rx_ui)?;
    // Session logic (timers).
    let session_status = session.start(tx_ui, rx_event);

    // Send termination to event handler if main thread encounters an error, in order to properly
    // shutdown the application.
    if session_status.is_err() {
        tx_termination.send(()).unwrap();
    }

    // Join threads.
    // It is very important to join `renderer_thread` in order to restore the terminal.
    renderer_thread.join().unwrap()?;
    event_handler_thread.join().unwrap()?;

    session_status
}

/// Run program, print message on error and return [`ExitCode`].
fn main() -> ExitCode {
    if let Err(err_msg) = run() {
        eprintln!("error: {err_msg}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
