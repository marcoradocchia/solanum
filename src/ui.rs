use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::Deserialize;
use std::{io, result, sync::mpsc::Receiver};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{self, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};
use crate::{Result, session::{Activity, TimerMessage}};

/// Setup terminal: initialize TUI.
pub fn setup_terminal() -> result::Result<Terminal<CrosstermBackend<io::Stdout>>, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    Ok(terminal)
}

/// Restore terminal.
pub fn restore_terminal(
    mut terminal: Terminal<CrosstermBackend<io::Stdout>>,
) -> result::Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

#[derive(Debug, Deserialize, Clone, Copy)]
/// UI colors.
#[serde(rename_all = "lowercase")]
pub enum Color {
    Red,
    Black,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
}

impl From<Color> for style::Color {
    fn from(color: Color) -> Self {
        match color {
            Color::Red => Self::Red,
            Color::Black => Self::Black,
            Color::Green => Self::Green,
            Color::Yellow => Self::Yellow,
            Color::Blue => Self::Blue,
            Color::Magenta => Self::Magenta,
            Color::Cyan => Self::Cyan,
            Color::Gray => Self::Gray,
            Color::DarkGray => Self::DarkGray,
            Color::LightRed => Self::LightRed,
            Color::LightGreen => Self::LightGreen,
            Color::LightYellow => Self::LightYellow,
            Color::LightBlue => Self::LightBlue,
            Color::LightMagenta => Self::LightMagenta,
            Color::LightCyan => Self::LightCyan,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
/// User Interface options, such as colors etc.
pub struct Ui {
    /// Pomodoro color.
    #[serde(default = "default_pomodoro_color")]
    pomodoro_color: Color,
    /// Short break color.
    #[serde(default = "default_short_break_color")]
    short_break_color: Color,
    /// Long break color.
    #[serde(default = "default_long_break_color")]
    long_break_color: Color,
    /// Progress bar background.
    #[serde(default = "default_background_color")]
    background_color: Color,
}

#[inline]
fn default_pomodoro_color() -> Color {
    Color::Green
}

#[inline]
fn default_short_break_color() -> Color {
    Color::Magenta
}

#[inline]
fn default_long_break_color() -> Color {
    Color::Red
}

#[inline]
fn default_background_color() -> Color {
    Color::DarkGray
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            pomodoro_color: default_pomodoro_color(),
            short_break_color: default_short_break_color(),
            long_break_color: default_long_break_color(),
            background_color: default_background_color(),
        }
    }
}

/// User Interface screens.
#[derive(Debug)]
pub enum Screen {
    Running,
    Pause,
    Expired,
}

impl Ui {
    pub fn render(
        &self,
        mut terminal: Terminal<CrosstermBackend<io::Stdout>>,
        rx: Receiver<TimerMessage>,
    ) -> Result<()> {
        for data in rx {
            let color: style::Color = match data.activity {
                Activity::Pomodoro(_) => self.pomodoro_color,
                Activity::ShortBreak => self.short_break_color,
                Activity::LongBreak => self.long_break_color,
            }.into();

            terminal
                .draw(|f| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(20), // Top empty space.
                            Constraint::Percentage(35), // Timer.
                            Constraint::Percentage(25), // Progress bar.
                            Constraint::Percentage(20), // Bottom empty space.
                        ])
                        .split(f.size());

                    let timer =
                        Paragraph::new(data.ascii)
                            .block(Block::default().borders(Borders::NONE))
                            .style(Style::default().fg(color))
                            .alignment(Alignment::Center);

                    let progress_bar = Gauge::default()
                        .block(
                            Block::default()
                                .borders(Borders::NONE)
                                .title_alignment(Alignment::Left)
                                .title(data.activity.to_string())
                                .border_style(Style::default().add_modifier(Modifier::BOLD)),
                        )
                        .gauge_style(
                            Style::default()
                                .fg(color)
                                .bg(self.background_color.into()),
                        )
                        .percent(data.perc as u16);

                    // Render widgets!
                    f.render_widget(timer, chunks[1]);
                    f.render_widget(progress_bar, chunks[2]);
                })
                .unwrap();

            // TODO: change this to be handled error.
            //
            // match screen {
            //     Screen::Running => Ok(()),
            //     Screen::Pause => Ok(()),
            //     Screen::Expired => Ok(()),
            // }
        }

        Ok(())
    }
}
