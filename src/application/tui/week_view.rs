use super::theme::Theme;
use crate::entities::{Journal, BulletType, TaskState};
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, poll};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::tty::IsTty;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use std::io::{self, Stdout, stdout};

#[derive(Debug, Clone)]
pub enum WeekViewResult {
    /// User exited without selecting (quit, escape, ctrl+c, etc.)
    Exited(NaiveDate),
    /// User selected a date to edit (pressed Enter)
    EditRequested(NaiveDate),
}

pub struct WeekView<'a> {
    /// Current week being focused (middle row)
    current_week_start: NaiveDate,
    /// Currently selected date
    selected_date: NaiveDate,
    /// Terminal instance
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Whether we should exit
    should_exit: bool,
    /// Whether user wants to edit the selected date
    should_edit: bool,
    /// Whether to show help text
    show_help: bool,
    /// Theme for styling
    theme: Theme,
    /// Journal reference for checking entries
    journal: &'a mut Journal,
}

impl<'a> WeekView<'a> {
    pub fn new(initial_date: NaiveDate, journal: &'a mut Journal) -> io::Result<Self> {
        // First check if we're in a proper terminal
        if !IsTty::is_tty(&std::io::stdout()) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Not running in a TTY, cannot initialize terminal interface",
            ));
        }

        enable_raw_mode().map_err(|e| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to enable raw mode: {}", e),
            )
        })?;

        stdout().execute(EnterAlternateScreen).map_err(|e| {
            let _ = disable_raw_mode(); // Clean up on failure
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to enter alternate screen: {}", e),
            )
        })?;

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend).map_err(|e| {
            let _ = disable_raw_mode();
            let _ = stdout().execute(LeaveAlternateScreen);
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to create terminal: {}", e),
            )
        })?;

        let week_start = Self::get_week_start(initial_date);

        Ok(Self {
            current_week_start: week_start,
            selected_date: initial_date,
            terminal,
            should_exit: false,
            should_edit: false,
            show_help: false,
            theme: Theme::default(),
            journal,
        })
    }

    /// Get the start of the week (Sunday) for a given date
    fn get_week_start(date: NaiveDate) -> NaiveDate {
        let days_since_sunday = match date.weekday() {
            Weekday::Sun => 0,
            Weekday::Mon => 1,
            Weekday::Tue => 2,
            Weekday::Wed => 3,
            Weekday::Thu => 4,
            Weekday::Fri => 5,
            Weekday::Sat => 6,
        };
        date - Duration::days(days_since_sunday as i64)
    }

    /// Generate dates for a week starting from the given Sunday
    fn get_week_dates(week_start: NaiveDate) -> Vec<NaiveDate> {
        (0..7).map(|i| week_start + Duration::days(i)).collect()
    }

    /// Check if a date has an entry in the journal
    fn has_entry(&mut self, date: NaiveDate) -> bool {
        self.journal.get_entry(date).unwrap_or(None).is_some()
    }

    /// Get entry status for all dates in the given range
    fn get_entry_statuses(
        &mut self,
        weeks: &[NaiveDate],
    ) -> std::collections::HashMap<NaiveDate, bool> {
        let mut statuses = std::collections::HashMap::new();

        for &week_start in weeks {
            let dates = Self::get_week_dates(week_start);
            for date in dates {
                let has_entry = self.journal.get_entry(date).unwrap_or(None).is_some();
                statuses.insert(date, has_entry);
            }
        }

        statuses
    }

    /// Calculate centered area with both horizontal and vertical centering
    fn calculate_centered_area(available: Rect, needed_width: u16, needed_height: u16) -> Rect {
        let width = std::cmp::min(available.width, needed_width);
        let height = std::cmp::min(available.height, needed_height);

        let left_margin = if available.width > width {
            (available.width - width) / 2
        } else {
            0
        };

        let top_margin = if available.height > height {
            (available.height - height) / 2
        } else {
            0
        };

        Rect {
            x: available.x + left_margin,
            y: available.y + top_margin,
            width,
            height,
        }
    }

    /// Get styling for a date cell based on various conditions (static version)
    fn get_date_style_static(
        date: NaiveDate,
        is_focused_week: bool,
        selected_date: NaiveDate,
        theme: &Theme,
    ) -> Style {
        let is_selected = date == selected_date;
        let is_today = date == chrono::Local::now().date_naive();
        let is_weekend = matches!(date.weekday(), Weekday::Sat | Weekday::Sun);

        if is_selected {
            // Subtle selection - slightly lighter background
            let light_bg = Color::Rgb(40, 40, 40);
            if is_weekend {
                Style::default().fg(theme.colors.weekend).bg(light_bg)
            } else if is_focused_week {
                Style::default().fg(theme.colors.focused).bg(light_bg)
            } else {
                Style::default().fg(theme.colors.dimmed).bg(light_bg)
            }
        } else if is_today {
            Style::default().fg(theme.colors.today).add_modifier(Modifier::BOLD)
        } else if is_weekend {
            Style::default().fg(theme.colors.weekend)
        } else if is_focused_week {
            Style::default().fg(theme.colors.focused)
        } else {
            Style::default().fg(theme.colors.dimmed)
        }
    }

    /// Create a table row for a week (static version)
    fn create_week_row_static(
        week_start: NaiveDate,
        is_focused: bool,
        selected_date: NaiveDate,
        theme: &Theme,
        entry_statuses: &std::collections::HashMap<NaiveDate, bool>,
    ) -> Row<'static> {
        let dates = Self::get_week_dates(week_start);
        let cells: Vec<Cell> = dates
            .iter()
            .map(|&date| {
                let day = date.day();
                let has_entry = *entry_statuses.get(&date).unwrap_or(&false);
                let _is_today = date == chrono::Local::now().date_naive();

                // Get base style (row style will handle background)
                let style = Self::get_date_style_static(date, is_focused, selected_date, theme);

                // Show month indicator on the 1st of each month
                let day_text = if day == 1 {
                    format!("{} {}", date.format("%b"), day)
                } else {
                    day.to_string()
                };

                // Add entry indicator (small dot)
                let content = if has_entry {
                    format!("{}•", day_text)
                } else {
                    format!("{} ", day_text) // space to keep alignment
                };

                Cell::from(content).style(style)
            })
            .collect();

        let mut row = Row::new(cells);

        // Apply focused week background to the entire row
        if is_focused {
            row = row.style(Style::default().bg(theme.colors.focused_week_bg));
        }

        row
    }

    /// Create the week view table (static version for drawing)
    fn create_week_table_static(
        current_week_start: NaiveDate,
        selected_date: NaiveDate,
        theme: &Theme,
        entry_statuses: &std::collections::HashMap<NaiveDate, bool>,
    ) -> Table<'static> {
        let focused_week = current_week_start;

        // Generate 5 weeks: 2 before, focused week, 2 after
        let weeks: Vec<NaiveDate> = (-2..=2)
            .map(|offset| focused_week + Duration::weeks(offset))
            .collect();

        let header = Row::new(vec![
            Cell::from("Sun").style(Style::default().fg(theme.colors.weekend)),
            Cell::from("Mon").style(Style::default().fg(theme.colors.header)),
            Cell::from("Tue").style(Style::default().fg(theme.colors.header)),
            Cell::from("Wed").style(Style::default().fg(theme.colors.header)),
            Cell::from("Thu").style(Style::default().fg(theme.colors.header)),
            Cell::from("Fri").style(Style::default().fg(theme.colors.header)),
            Cell::from("Sat").style(Style::default().fg(theme.colors.weekend)),
        ])
        .height(1);

        let rows: Vec<Row> = weeks
            .iter()
            .enumerate()
            .map(|(i, &week_start)| {
                let is_focused = i == 2; // Middle row (index 2) is focused
                Self::create_week_row_static(
                    week_start,
                    is_focused,
                    selected_date,
                    theme,
                    entry_statuses,
                )
            })
            .collect();

        Table::new(
            rows,
            [
                Constraint::Percentage(14), // ~14.3% each for 7 columns
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(14),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .title(format!("{}", selected_date.format("%B %Y")))
                .title_style(Style::default().fg(theme.colors.header))
                .title_alignment(Alignment::Center),
        )
        .column_spacing(1)
    }

    /// Handle keyboard input
    fn handle_key_event(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            // Exit
            (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => {
                self.should_exit = true;
            }

            // Ctrl+C and Ctrl+D
            (KeyCode::Char('c'), KeyModifiers::CONTROL)
            | (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                self.should_exit = true;
            }

            // Navigation - Arrow keys
            (KeyCode::Left, _) | (KeyCode::Char('h'), _) => {
                self.selected_date = self.selected_date - Duration::days(1);
                self.update_current_week();
            }
            (KeyCode::Right, _) | (KeyCode::Char('l'), _) => {
                self.selected_date = self.selected_date + Duration::days(1);
                self.update_current_week();
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                self.selected_date = self.selected_date - Duration::weeks(1);
                self.update_current_week();
            }
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                self.selected_date = self.selected_date + Duration::weeks(1);
                self.update_current_week();
            }

            // Jump to today
            (KeyCode::Char('t'), _) => {
                self.selected_date = chrono::Local::now().date_naive();
                self.update_current_week();
            }

            // Enter to edit selected date
            (KeyCode::Enter, _) => {
                self.should_edit = true;
                self.should_exit = true;
            }

            // Toggle help
            (KeyCode::Char('?'), _) => {
                self.show_help = !self.show_help;
            }

            // Jump by month
            (KeyCode::PageUp, _) => {
                self.selected_date = self.selected_date - Duration::days(30);
                self.update_current_week();
            }
            (KeyCode::PageDown, _) => {
                self.selected_date = self.selected_date + Duration::days(30);
                self.update_current_week();
            }

            _ => {}
        }
    }

    /// Update the current week focus based on selected date
    fn update_current_week(&mut self) {
        let selected_week_start = Self::get_week_start(self.selected_date);

        // Only update if we've moved to a different week
        if selected_week_start != self.current_week_start {
            self.current_week_start = selected_week_start;
        }
    }

    /// Create help text (static version)
    fn create_help_text_static(selected_date: NaiveDate, theme: &Theme) -> Paragraph<'static> {
        let help_text = vec![
            Line::from(vec![Span::styled(
                "↑↓/jk=Week • ←→/hl=Day • PgUp/PgDn=Month • t=Today • Enter=Edit • ?=Help • q=Quit",
                Style::default().fg(theme.colors.dimmed),
            )]),
            Line::from(vec![Span::styled(
                format!("{}", selected_date.format("%A, %B %d, %Y")),
                Style::default().fg(theme.colors.focused),
            )]),
        ];

        Paragraph::new(help_text)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center)
    }

    /// Create bullet display widget for the selected date
    fn create_bullet_display(
        entry: Option<&crate::entities::Entry>,
        theme: &Theme,
    ) -> Paragraph<'static> {
        let entry = match entry {
            Some(entry) => entry,
            None => {
                return Paragraph::new(vec![Line::from(vec![Span::styled(
                    "No entry for this date".to_string(),
                    Style::default().fg(theme.colors.dimmed),
                )])])
                .block(Block::default().borders(Borders::NONE))
                .alignment(Alignment::Left)
            }
        };

        let mut lines = Vec::new();
        
        let bullet_types = [
            BulletType::Task,
            BulletType::Event,
            BulletType::Note,
            BulletType::Priority,
            BulletType::Inspiration,
            BulletType::Insight,
            BulletType::Misstep,
        ];

        for bullet_type in bullet_types {
            let bullets = entry.get_bullets(&bullet_type);
            for bullet in bullets {
                let symbol = bullet.symbol();

                let bullet_style = match bullet_type {
                    BulletType::Priority => Style::default().fg(Color::Yellow),
                    BulletType::Task if bullet.task_state == Some(TaskState::Completed) => {
                        Style::default().fg(Color::Green)
                    }
                    BulletType::Inspiration => Style::default().fg(Color::Cyan),
                    BulletType::Insight => Style::default().fg(Color::Magenta),
                    BulletType::Misstep => Style::default().fg(Color::Red),
                    _ => Style::default().fg(theme.colors.focused),
                };

                lines.push(Line::from(vec![
                    Span::styled(format!("{} ", symbol), bullet_style),
                    Span::styled(bullet.content.clone(), Style::default().fg(theme.colors.focused)),
                ]));
            }
        }

        if lines.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                "No bullets for this date".to_string(),
                Style::default().fg(theme.colors.dimmed),
            )]));
        }

        Paragraph::new(lines)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Left)
    }

    /// Run the week view TUI loop
    pub fn run(&mut self) -> io::Result<WeekViewResult> {
        loop {
            // Check for exit condition before drawing
            if self.should_exit {
                break;
            }

            // Generate weeks we need to check for entry statuses
            let weeks: Vec<NaiveDate> = (-2..=2)
                .map(|offset| self.current_week_start + Duration::weeks(offset))
                .collect();

            // Get entry statuses before drawing (requires mutable access to journal)
            let entry_statuses = self.get_entry_statuses(&weeks);
            
            // Get the selected date's entry for bullet display
            let selected_entry = self.journal.get_entry(self.selected_date)
                .unwrap_or(None)
                .cloned();

            // Capture the state we need for drawing (after mutable borrow is complete)
            let current_week_start = self.current_week_start;
            let selected_date = self.selected_date;
            let show_help = self.show_help;
            let theme = &self.theme;

            self.terminal.draw(|frame| {
                let size = frame.area();

                // Calculate the total space needed for our UI
                const CALENDAR_HEIGHT: u16 = 18; // 5 weeks * 3 rows each + header + title
                const HELP_HEIGHT: u16 = 3; // Help text (no borders)
                const BULLET_HEIGHT: u16 = 8; // Space for bullet display

                const MIN_CALENDAR_WIDTH: u16 = 78;
                const MAX_CALENDAR_WIDTH: u16 = 100;
                const PREFERRED_CALENDAR_WIDTH: u16 = 86;

                let needed_width = if size.width >= MAX_CALENDAR_WIDTH + 10 {
                    PREFERRED_CALENDAR_WIDTH
                } else if size.width >= MIN_CALENDAR_WIDTH + 4 {
                    std::cmp::min(size.width.saturating_sub(4), MAX_CALENDAR_WIDTH)
                } else {
                    std::cmp::min(size.width, MIN_CALENDAR_WIDTH)
                };

                let total_height = if show_help {
                    CALENDAR_HEIGHT + BULLET_HEIGHT + HELP_HEIGHT
                } else {
                    CALENDAR_HEIGHT + BULLET_HEIGHT
                };

                // Calculate centered area for the entire UI
                let centered_area = Self::calculate_centered_area(size, needed_width, total_height);

                // Create and draw week table
                let table = Self::create_week_table_static(
                    current_week_start,
                    selected_date,
                    theme,
                    &entry_statuses,
                );

                if show_help {
                    // Create vertical layout within the centered area
                    let main_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(CALENDAR_HEIGHT), // Main week view
                            Constraint::Length(BULLET_HEIGHT),   // Bullet display
                            Constraint::Length(HELP_HEIGHT),     // Help text
                        ])
                        .split(centered_area);

                    frame.render_widget(table, main_chunks[0]);
                    
                    // Create and draw bullet display
                    let bullet_display = Self::create_bullet_display(selected_entry.as_ref(), theme);
                    frame.render_widget(bullet_display, main_chunks[1]);

                    // Create and draw help
                    let help = Self::create_help_text_static(selected_date, theme);
                    frame.render_widget(help, main_chunks[2]);
                } else {
                    // Create vertical layout for calendar and bullets
                    let main_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(CALENDAR_HEIGHT), // Main week view
                            Constraint::Length(BULLET_HEIGHT),   // Bullet display
                        ])
                        .split(centered_area);

                    frame.render_widget(table, main_chunks[0]);
                    
                    // Create and draw bullet display
                    let bullet_display = Self::create_bullet_display(selected_entry.as_ref(), theme);
                    frame.render_widget(bullet_display, main_chunks[1]);
                }
            })?;

            // Handle events with timeout to prevent blocking indefinitely
            match poll(std::time::Duration::from_millis(100))? {
                true => {
                    match event::read()? {
                        Event::Key(key) => {
                            self.handle_key_event(key);
                        }
                        Event::Resize(_, _) => {
                            // Terminal was resized, continue to redraw
                            continue;
                        }
                        _ => {
                            // Other events (mouse, focus, etc.) - ignore
                            continue;
                        }
                    }
                }
                false => {
                    // No event received, continue loop
                    continue;
                }
            }
        }

        // Ensure proper cleanup before returning
        self.cleanup()?;

        // Return result based on user action
        if self.should_edit {
            Ok(WeekViewResult::EditRequested(self.selected_date))
        } else {
            Ok(WeekViewResult::Exited(self.selected_date))
        }
    }

    /// Explicit cleanup method
    fn cleanup(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        self.terminal.backend_mut().execute(LeaveAlternateScreen)?;
        Ok(())
    }
}

impl<'a> Drop for WeekView<'a> {
    fn drop(&mut self) {
        // Fallback cleanup if explicit cleanup wasn't called
        let _ = self.cleanup();
    }
}
