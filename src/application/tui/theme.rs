use crossterm::style::{Color as CrosstermColor, Stylize};
use ratatui::style::{Color as RatatuiColor, Style as RatatuiStyle, Modifier};

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    // Primary colors
    pub primary: ColorPair,
    pub secondary: ColorPair,
    pub accent: ColorPair,
    
    // State colors
    pub selected: ColorPair,
    pub today: ColorPair,
    pub focused: ColorPair,
    pub focused_week_bg: ColorPair,
    pub dimmed: ColorPair,
    
    // UI elements
    pub border: ColorPair,
    pub header: ColorPair,
    pub weekend: ColorPair,
    pub month_indicator: ColorPair,
    
    // Text
    pub normal_text: ColorPair,
    pub help_text: ColorPair,
    pub error_text: ColorPair,
}

#[derive(Debug, Clone)]
pub struct ColorPair {
    pub fg: CrosstermColor,
    pub bg: Option<CrosstermColor>,
}

impl ColorPair {
    pub fn new(fg: CrosstermColor) -> Self {
        Self { fg, bg: None }
    }
    
    pub fn with_bg(fg: CrosstermColor, bg: CrosstermColor) -> Self {
        Self { fg, bg: Some(bg) }
    }
    
    /// Convert to ratatui style
    pub fn to_ratatui(&self) -> RatatuiStyle {
        let mut style = RatatuiStyle::default().fg(self.crossterm_to_ratatui(self.fg));
        if let Some(bg) = self.bg {
            style = style.bg(self.crossterm_to_ratatui(bg));
        }
        style
    }
    
    /// Convert crossterm color to ratatui color
    pub fn crossterm_to_ratatui(&self, color: CrosstermColor) -> RatatuiColor {
        match color {
            CrosstermColor::Black => RatatuiColor::Black,
            CrosstermColor::DarkRed => RatatuiColor::Red,
            CrosstermColor::DarkGreen => RatatuiColor::Green,
            CrosstermColor::DarkYellow => RatatuiColor::Yellow,
            CrosstermColor::DarkBlue => RatatuiColor::Blue,
            CrosstermColor::DarkMagenta => RatatuiColor::Magenta,
            CrosstermColor::DarkCyan => RatatuiColor::Cyan,
            CrosstermColor::Grey => RatatuiColor::Gray,
            CrosstermColor::DarkGrey => RatatuiColor::DarkGray,
            CrosstermColor::Red => RatatuiColor::LightRed,
            CrosstermColor::Green => RatatuiColor::LightGreen,
            CrosstermColor::Yellow => RatatuiColor::LightYellow,
            CrosstermColor::Blue => RatatuiColor::LightBlue,
            CrosstermColor::Magenta => RatatuiColor::LightMagenta,
            CrosstermColor::Cyan => RatatuiColor::LightCyan,
            CrosstermColor::White => RatatuiColor::White,
            CrosstermColor::Rgb { r, g, b } => RatatuiColor::Rgb(r, g, b),
            CrosstermColor::AnsiValue(v) => RatatuiColor::Indexed(v),
            _ => RatatuiColor::White,
        }
    }
    
    /// Convert to ratatui style with modifier
    pub fn to_ratatui_with_modifier(&self, modifier: Modifier) -> RatatuiStyle {
        self.to_ratatui().add_modifier(modifier)
    }
    
    /// Apply color to text for terminal output
    pub fn colorize(&self, text: &str) -> String {
        let mut styled = text.with(self.fg);
        if let Some(bg) = self.bg {
            styled = styled.on(bg);
        }
        format!("{}", styled)
    }
    
    /// Apply color with bold
    pub fn colorize_bold(&self, text: &str) -> String {
        let mut styled = text.with(self.fg).bold();
        if let Some(bg) = self.bg {
            styled = styled.on(bg);
        }
        format!("{}", styled)
    }
}


impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            colors: ThemeColors {
                primary: ColorPair::new(CrosstermColor::White),
                secondary: ColorPair::new(CrosstermColor::Grey),
                accent: ColorPair::new(CrosstermColor::Blue),
                
                selected: ColorPair::new(CrosstermColor::Blue),
                today: ColorPair::with_bg(CrosstermColor::Black, CrosstermColor::Yellow),
                focused: ColorPair::new(CrosstermColor::White),
                focused_week_bg: ColorPair::with_bg(CrosstermColor::White, CrosstermColor::Rgb { r: 28, g: 28, b: 28 }),
                dimmed: ColorPair::new(CrosstermColor::DarkGrey),
                
                border: ColorPair::new(CrosstermColor::DarkCyan),
                header: ColorPair::new(CrosstermColor::DarkCyan),
                weekend: ColorPair::new(CrosstermColor::Rgb { r: 150, g: 150, b: 150 }),
                month_indicator: ColorPair::new(CrosstermColor::Green),
                
                normal_text: ColorPair::new(CrosstermColor::White),
                help_text: ColorPair::new(CrosstermColor::DarkCyan),
                error_text: ColorPair::new(CrosstermColor::Red),
            },
        }
    }
    
    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            colors: ThemeColors {
                primary: ColorPair::new(CrosstermColor::Black),
                secondary: ColorPair::new(CrosstermColor::DarkGrey),
                accent: ColorPair::new(CrosstermColor::DarkBlue),
                
                selected: ColorPair::with_bg(CrosstermColor::White, CrosstermColor::DarkBlue),
                today: ColorPair::with_bg(CrosstermColor::Black, CrosstermColor::Yellow),
                focused: ColorPair::new(CrosstermColor::Black),
                focused_week_bg: ColorPair::with_bg(CrosstermColor::Black, CrosstermColor::Rgb { r: 240, g: 240, b: 240 }),
                dimmed: ColorPair::new(CrosstermColor::Grey),
                
                border: ColorPair::new(CrosstermColor::DarkBlue),
                header: ColorPair::new(CrosstermColor::DarkBlue),
                weekend: ColorPair::new(CrosstermColor::Grey),
                month_indicator: ColorPair::new(CrosstermColor::DarkGreen),
                
                normal_text: ColorPair::new(CrosstermColor::Black),
                help_text: ColorPair::new(CrosstermColor::DarkBlue),
                error_text: ColorPair::new(CrosstermColor::DarkRed),
            },
        }
    }
    
    pub fn minimal() -> Self {
        Self {
            name: "Minimal".to_string(),
            colors: ThemeColors {
                primary: ColorPair::new(CrosstermColor::White),
                secondary: ColorPair::new(CrosstermColor::Grey),
                accent: ColorPair::new(CrosstermColor::White),
                
                selected: ColorPair::new(CrosstermColor::White),
                today: ColorPair::new(CrosstermColor::White),
                focused: ColorPair::new(CrosstermColor::White),
                focused_week_bg: ColorPair::with_bg(CrosstermColor::White, CrosstermColor::Rgb { r: 28, g: 28, b: 28 }),
                dimmed: ColorPair::new(CrosstermColor::DarkGrey),
                
                border: ColorPair::new(CrosstermColor::White),
                header: ColorPair::new(CrosstermColor::White),
                weekend: ColorPair::new(CrosstermColor::DarkGrey),
                month_indicator: ColorPair::new(CrosstermColor::White),
                
                normal_text: ColorPair::new(CrosstermColor::White),
                help_text: ColorPair::new(CrosstermColor::Grey),
                error_text: ColorPair::new(CrosstermColor::White),
            },
        }
    }
    
    pub fn by_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "light" => Self::light(),
            "minimal" => Self::minimal(),
            _ => Self::dark(), // default
        }
    }
    
    pub fn available_themes() -> Vec<String> {
        vec!["dark".to_string(), "light".to_string(), "minimal".to_string()]
    }
}