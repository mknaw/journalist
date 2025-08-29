use crate::application::JournalApp;
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
            None => {
                // Default: start TUI
                app.run_tui()?;
            }
        }

        Ok(())
    }
}
