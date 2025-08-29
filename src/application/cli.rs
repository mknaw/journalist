use crate::application::{JournalApp, WeekView, WeekViewResult, test_terminal_setup};
use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "journalist")]
#[command(about = "A terminal-based bullet journal application")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create/edit today's entry (opens $EDITOR)
    New {
        /// Specific date (YYYY-MM-DD format, defaults to today)
        #[arg(short, long)]
        date: Option<String>,
    },
    /// Start the interactive TUI
    Tui,
    /// Start the week view TUI
    Week {
        /// Specific date to focus on (YYYY-MM-DD format, defaults to today)
        #[arg(short, long)]
        date: Option<String>,
    },
    /// Test terminal capabilities
    TestTerminal,
}

impl Cli {
    pub fn run() -> anyhow::Result<()> {
        let cli = Self::parse();
        let mut app = JournalApp::new();

        match cli.command {
            Some(Commands::New { date }) => {
                let target_date = if let Some(date_str) = date {
                    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")?
                } else {
                    Local::now().naive_local().date()
                };

                app.edit_entry_for_date(target_date)?;
            }
            Some(Commands::Tui) => {
                app.run_tui()?;
            }
            Some(Commands::TestTerminal) => {
                test_terminal_setup()?;
            }
            Some(Commands::Week { date }) => {
                let target_date = if let Some(date_str) = date {
                    NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")?
                } else {
                    Local::now().naive_local().date()
                };

                let mut week_view = WeekView::new(target_date)?;
                match week_view.run()? {
                    WeekViewResult::EditRequested(selected_date) => {
                        app.edit_entry_for_date(selected_date)?;
                    }
                    WeekViewResult::Exited(_) => {
                        // User exited without selecting, do nothing
                    }
                }
            }
            None => {
                // Default: start week view
                let target_date = Local::now().naive_local().date();
                
                let mut week_view = WeekView::new(target_date)?;
                match week_view.run()? {
                    WeekViewResult::EditRequested(selected_date) => {
                        app.edit_entry_for_date(selected_date)?;
                    }
                    WeekViewResult::Exited(_) => {
                        // User exited without selecting, do nothing
                    }
                }
            }
        }

        Ok(())
    }
}
