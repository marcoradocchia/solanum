mod error;
mod session;
mod ascii;
mod timer;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use error::Error;
use session::Session;
use timer::Timer;
use std::{io, process, str::FromStr};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
    Terminal,
};
use ascii::Ascii;

pub type Result<T> = std::result::Result<T, Error>;

/// Setup terminal: initialize TUI.
fn setup_terminal() -> std::result::Result<Terminal<CrosstermBackend<io::Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    Ok(terminal)
}

/// Restore terminal to previous screen.
fn restore_terminal(
    mut terminal: Terminal<CrosstermBackend<io::Stdout>>,
) -> std::result::Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

// Run and collect errors.
fn run(mut session: Session) -> Result<()> {
    let mut terminal = setup_terminal().map_err(Error::Terminal)?;

    terminal
        .draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20), // Top empty space.
                    Constraint::Percentage(30), // Timer.
                    Constraint::Percentage(20), // Progress bar.
                    Constraint::Percentage(10), // Pomodoro/break counter.
                    Constraint::Percentage(20), // Bottom empty space.
                ])
                .split(f.size());

            let timer = Paragraph::new("ciao")
                .block(Block::default().borders(Borders::NONE))
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center);

            let progress_bar = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title_alignment(Alignment::Center)
                        .title("Solanum")
                        .border_type(BorderType::Rounded),
                )
                .gauge_style(Style::default().fg(Color::Yellow))
                .percent(20);

            // Render time!
            f.render_widget(timer, chunks[1]);
            f.render_widget(progress_bar, chunks[2]);
        })
        .unwrap();

    // Start `solanum` session.
    // session.start(&mut terminal);
    std::thread::sleep(std::time::Duration::from_secs(5));

    restore_terminal(terminal).map_err(Error::Terminal)?;

    Ok(())
}

fn main() {
    // Parse CLI arguments and instantiate a new [`Session`].
    let session = Session::cli();

    if let Err(err_msg) = run(session) {
        eprintln!("error: {err_msg}");
        process::exit(1);
    }
}
