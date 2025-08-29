use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    tty::IsTty,
    ExecutableCommand,
};
use std::io::{self, stdout};

pub fn test_terminal_setup() -> io::Result<()> {
    println!("Testing terminal capabilities...");
    
    // Check if we're in a TTY
    if IsTty::is_tty(&stdout()) {
        println!("✓ Running in a TTY");
    } else {
        println!("✗ Not running in a TTY");
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Terminal interface requires a TTY",
        ));
    }
    
    // Test raw mode
    print!("Testing raw mode... ");
    enable_raw_mode()?;
    println!("✓ Raw mode enabled");
    
    // Test alternate screen
    print!("Testing alternate screen... ");
    stdout().execute(EnterAlternateScreen)?;
    println!("✓ Alternate screen enabled");
    
    // Clean up
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    println!("✓ Terminal restored");
    
    println!("All terminal tests passed!");
    Ok(())
}