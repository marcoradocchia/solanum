use crate::{
    error::Error,
    session::Activity,
    timer::{TimerData, TimerStatus},
    Result,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::Deserialize;
use std::{io, result, sync::mpsc::Receiver, thread};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{self, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};

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

/// User interface commands.
#[derive(Debug, Clone)]
pub enum UiCommand {
    Draw(TimerStatus),
    Refresh,
}

/// User interface screens.
#[derive(Debug, Clone, Default)]
pub enum Screen {
    #[default]
    Running,
    Paused,
    Expired,
}

#[derive(Debug, Deserialize, Clone, Copy)]
/// User Interface options, such as colors etc.
pub struct UiOptions {
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

impl Default for UiOptions {
    fn default() -> Self {
        Self {
            pomodoro_color: default_pomodoro_color(),
            short_break_color: default_short_break_color(),
            long_break_color: default_long_break_color(),
            background_color: default_background_color(),
        }
    }
}

/// User Interface.
#[derive(Debug, Clone)]
pub struct Ui {
    /// User Interface options.
    options: UiOptions,
    /// Current timer data.
    timer_data: TimerData,
    /// Current screen.
    screen: Screen,
}

impl Ui {
    /// Construct new instance.
    pub fn new(options: UiOptions) -> Self {
        Self {
            options,
            timer_data: Default::default(),
            screen: Default::default(),
        }
    }

    /// Render timer screen (whether running or paused).
    fn render_timer(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        terminal
            .draw(|frame| {
                let color = match self.timer_data.activity {
                    Activity::Pomodoro(_) => self.options.pomodoro_color,
                    Activity::ShortBreak => self.options.short_break_color,
                    Activity::LongBreak => self.options.long_break_color,
                };

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(20), // Top empty space.
                        Constraint::Percentage(35), // Timer.
                        Constraint::Percentage(25), // Progress bar.
                        Constraint::Percentage(20), // Bottom empty space.
                    ])
                    .split(frame.size());

                let timer = Paragraph::new(self.timer_data.ascii.as_ref())
                    .block(Block::default().borders(Borders::NONE))
                    .style(Style::default().fg(color.into()))
                    .alignment(Alignment::Center);

                let progress_bar = Gauge::default()
                    .block(
                        Block::default()
                            .borders(Borders::NONE)
                            .title_alignment(Alignment::Left)
                            .title(self.timer_data.activity.to_string())
                            .border_style(Style::default().add_modifier(Modifier::BOLD)),
                    )
                    .gauge_style(
                        Style::default()
                            .fg(color.into())
                            .bg(self.options.background_color.into()),
                    )
                    .percent(self.timer_data.perc);

                // Render widgets!
                frame.render_widget(timer, layout[1]);
                frame.render_widget(progress_bar, layout[2]);
            })
            .map_err(Error::Terminal)?;

        Ok(())
    }

    /// Render expired screen.
    fn render_expired(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        terminal
            .draw(|frame| {
                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(45), // Top empty space.
                        Constraint::Percentage(10), // Timer.
                        Constraint::Percentage(45), // Bottom empty space.
                    ])
                    .split(frame.size());

                let text = Paragraph::new("Timer expired")
                    .block(Block::default().borders(Borders::NONE))
                    .alignment(Alignment::Center);

                frame.render_widget(text, layout[1]);
            })
            .map_err(Error::Terminal)?;

        Ok(())
    }

    fn draw_screen(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        match self.screen {
            Screen::Running => self.render_timer(terminal)?,
            Screen::Paused => self.render_timer(terminal)?,
            Screen::Expired => self.render_expired(terminal)?,
        }

        Ok(())
    }

    /// Spawn thread listening for [`UiCommand`]s.
    pub fn spawn_thread(
        mut self,
        rx: Receiver<UiCommand>,
    ) -> Result<thread::JoinHandle<Result<()>>> {
        // Setup terminal for TUI.
        let mut terminal = setup_terminal().map_err(Error::Terminal)?;

        Ok(thread::spawn(move || {
            for ui_command in rx {
                match ui_command {
                    UiCommand::Draw(timer_status) => {
                        // Update current screen.
                        self.screen = match timer_status {
                            TimerStatus::Running(timer_data) => {
                                // Update timer data.
                                self.timer_data = timer_data;
                                Screen::Running
                            }
                            TimerStatus::Paused => Screen::Paused,
                            TimerStatus::Expired => Screen::Expired,
                        };
                        // Draw the screen
                        self.draw_screen(&mut terminal)?;
                    }
                    UiCommand::Refresh => self.draw_screen(&mut terminal)?,
                }
            }

            // Restore terminal to previous screen and behaviour.
            restore_terminal(terminal).map_err(Error::Terminal)?;

            Ok(())
        }))
    }
}
