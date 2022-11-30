use crate::{ui::UiCommand, Result};
use crossterm::event::{self, read, KeyCode::Char, KeyEventKind, KeyEventState, KeyModifiers};
use std::{
    sync::mpsc::{Sender, Receiver},
    thread::{self, JoinHandle},
};

/// List of application events.
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    TogglePause,
    Skip,
}

pub struct EventHandler();

impl EventHandler {
    /// Spawn event handler on new trhead.
    pub fn spawn_thread(
        tx_event: Sender<Event>,
        tx_ui: Sender<UiCommand>,
        rx_termination: Receiver<()>
    ) -> JoinHandle<Result<()>> {
        thread::spawn(move || -> Result<()> {
            while rx_termination.try_recv().is_err() {
                match read()? {
                    event::Event::Key(key_event) => {
                        // Ignore keyboad events which are not press or are not simple key press.
                        if key_event.modifiers != KeyModifiers::NONE
                            || key_event.state != KeyEventState::NONE
                            || key_event.kind != KeyEventKind::Press
                        {
                            continue;
                        }

                        match key_event.code {
                            // Pause timer.
                            Char('p') | Char(' ') => tx_event.send(Event::TogglePause).unwrap(),
                            // Skip current timer.
                            Char('s') => tx_event.send(Event::Skip).unwrap(),
                            // Quit application.
                            Char('q') => return Ok(()),
                            _ => continue,
                        }
                    }
                    event::Event::Resize(_, _) => tx_ui.send(UiCommand::Refresh).unwrap(),
                    _ => {} // Ignoring other event types.
                }
            }

            Ok(())
        })
    }
}
