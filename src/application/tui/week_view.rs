use super::theme::Theme;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers, poll};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::tty::IsTty;
use crossterm::ExecutableCommand;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Terminal,
};
use std::io::{self, stdout, Stdout};

#[derive(Debug, Clone)]
pub enum WeekViewResult {
    /// User exited without selecting (quit, escape, ctrl+c, etc.)
    Exited(NaiveDate),
    /// User selected a date to edit (pressed Enter)
    EditRequested(NaiveDate),
}

pub struct WeekView {
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
}

impl WeekView {
    pub fn new(initial_date: NaiveDate) -> io::Result<Self> {
        // First check if we're in a proper terminal
        if !IsTty::is_tty(&std::io::stdout()) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Not running in a TTY, cannot initialize terminal interface",
            ));
        }
        
        enable_raw_mode()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to enable raw mode: {}", e)))?;
        
        stdout().execute(EnterAlternateScreen)
            .map_err(|e| {
                let _ = disable_raw_mode(); // Clean up on failure
                io::Error::new(io::ErrorKind::Other, format!("Failed to enter alternate screen: {}", e))
            })?;
        
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)
            .map_err(|e| {
                let _ = disable_raw_mode();
                let _ = stdout().execute(LeaveAlternateScreen);
                io::Error::new(io::ErrorKind::Other, format!("Failed to create terminal: {}", e))
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
    
    /// Mock function to check if a date has an entry (randomly half will have entries)
    fn has_entry(date: NaiveDate) -> bool {
        // Use date as seed for consistent "random" results
        let seed = (date.year() as u32 * 10000 + date.month() * 100 + date.day()) % 2;
        seed == 0
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
    fn get_date_style_static(date: NaiveDate, is_focused_week: bool, selected_date: NaiveDate, theme: &Theme) -> Style {
        let is_selected = date == selected_date;
        let is_today = date == chrono::Local::now().date_naive();
        let is_weekend = matches!(date.weekday(), Weekday::Sat | Weekday::Sun);
        
        if is_selected {
            // Subtle selection - just underlined
            if is_weekend {
                theme.colors.weekend.to_ratatui_with_modifier(Modifier::UNDERLINED)
            } else if is_focused_week {
                theme.colors.focused.to_ratatui_with_modifier(Modifier::UNDERLINED)
            } else {
                theme.colors.dimmed.to_ratatui_with_modifier(Modifier::UNDERLINED)
            }
        } else if is_today {
            theme.colors.today.to_ratatui_with_modifier(Modifier::BOLD)
        } else if is_weekend {
            theme.colors.weekend.to_ratatui()
        } else if is_focused_week {
            theme.colors.focused.to_ratatui()
        } else {
            theme.colors.dimmed.to_ratatui()
        }
    }
    
    /// Create a table row for a week (static version)
    fn create_week_row_static(week_start: NaiveDate, is_focused: bool, selected_date: NaiveDate, theme: &Theme) -> Row<'static> {
        let dates = Self::get_week_dates(week_start);
        let cells: Vec<Cell> = dates
            .iter()
            .map(|&date| {
                let day = date.day();
                let has_entry = Self::has_entry(date);
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
        
        let mut row = Row::new(cells).height(3); // More vertical space for centered feel
        
        // Apply focused week background to the entire row
        if is_focused {
            row = row.style(theme.colors.focused_week_bg.to_ratatui());
        }
        
        row
    }
    
    /// Create the week view table (static version for drawing)
    fn create_week_table_static(current_week_start: NaiveDate, selected_date: NaiveDate, theme: &Theme) -> Table<'static> {
        let focused_week = current_week_start;
        
        // Generate 5 weeks: 2 before, focused week, 2 after
        let weeks: Vec<NaiveDate> = (-2..=2)
            .map(|offset| focused_week + Duration::weeks(offset))
            .collect();
        
        let header = Row::new(vec![
            Cell::from("Sun").style(theme.colors.weekend.to_ratatui()),
            Cell::from("Mon").style(theme.colors.header.to_ratatui()),
            Cell::from("Tue").style(theme.colors.header.to_ratatui()),
            Cell::from("Wed").style(theme.colors.header.to_ratatui()),
            Cell::from("Thu").style(theme.colors.header.to_ratatui()),
            Cell::from("Fri").style(theme.colors.header.to_ratatui()),
            Cell::from("Sat").style(theme.colors.weekend.to_ratatui()),
        ]).height(1);
        
        let rows: Vec<Row> = weeks
            .iter()
            .enumerate()
            .map(|(i, &week_start)| {
                let is_focused = i == 2; // Middle row (index 2) is focused
                Self::create_week_row_static(week_start, is_focused, selected_date, theme)
            })
            .collect();
        
        Table::new(rows, [
            Constraint::Percentage(14), // ~14.3% each for 7 columns
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .title(format!("{}", selected_date.format("%B %Y")))
                .title_style(theme.colors.header.to_ratatui())
                .title_alignment(Alignment::Center)
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
            (KeyCode::Char('c'), KeyModifiers::CONTROL) | (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
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
            Line::from(vec![
                Span::styled("↑↓/jk=Week • ←→/hl=Day • PgUp/PgDn=Month • t=Today • Enter=Edit • ?=Help • q=Quit", theme.colors.dimmed.to_ratatui()),
            ]),
            Line::from(vec![
                Span::styled(format!("{}", selected_date.format("%A, %B %d, %Y")), theme.colors.focused.to_ratatui()),
            ]),
        ];
        
        Paragraph::new(help_text)
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Center)
    }
    
    /// Run the week view TUI loop
    pub fn run(&mut self) -> io::Result<WeekViewResult> {
        loop {
            // Check for exit condition before drawing
            if self.should_exit {
                break;
            }
            
            // Capture the state we need for drawing
            let current_week_start = self.current_week_start;
            let selected_date = self.selected_date;
            let show_help = self.show_help;
            let theme = &self.theme;
            
            self.terminal.draw(|frame| {
                let size = frame.area();
                
                // Calculate the total space needed for our UI
                const CALENDAR_HEIGHT: u16 = 18; // 5 weeks * 3 rows each + header + title
                const HELP_HEIGHT: u16 = 3;      // Help text (no borders)
                
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
                    CALENDAR_HEIGHT + HELP_HEIGHT
                } else {
                    CALENDAR_HEIGHT
                };
                
                // Calculate centered area for the entire UI
                let centered_area = Self::calculate_centered_area(size, needed_width, total_height);
                
                // Create and draw week table
                let table = Self::create_week_table_static(current_week_start, selected_date, theme);
                
                if show_help {
                    // Create vertical layout within the centered area
                    let main_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Length(CALENDAR_HEIGHT), // Main week view
                            Constraint::Length(HELP_HEIGHT),     // Help text
                        ])
                        .split(centered_area);
                    
                    frame.render_widget(table, main_chunks[0]);
                    
                    // Create and draw help
                    let help = Self::create_help_text_static(selected_date, theme);
                    frame.render_widget(help, main_chunks[1]);
                } else {
                    // Just draw the calendar
                    frame.render_widget(table, centered_area);
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

impl Drop for WeekView {
    fn drop(&mut self) {
        // Fallback cleanup if explicit cleanup wasn't called
        let _ = self.cleanup();
    }
}