use ratatui::style::Color;

#[derive(Debug, Clone)]
pub struct Theme {
    pub colors: ThemeColors,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    // Primary colors
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    
    // State colors
    pub selected: Color,
    pub today: Color,
    pub focused: Color,
    pub focused_week_bg: Color,
    pub dimmed: Color,
    
    // UI elements
    pub border: Color,
    pub header: Color,
    pub weekend: Color,
    pub month_indicator: Color,
    
    // Text
    pub normal_text: Color,
    pub help_text: Color,
    pub error_text: Color,
}



impl Default for Theme {
    fn default() -> Self {
        Self {
            colors: ThemeColors {
                primary: Color::White,
                secondary: Color::Gray,
                accent: Color::Blue,
                
                selected: Color::Blue,
                today: Color::Yellow,
                focused: Color::White,
                focused_week_bg: Color::Rgb(28, 28, 28),
                dimmed: Color::DarkGray,
                
                border: Color::Cyan,
                header: Color::Cyan,
                weekend: Color::Rgb(150, 150, 150),
                month_indicator: Color::Green,
                
                normal_text: Color::White,
                help_text: Color::Cyan,
                error_text: Color::Red,
            },
        }
    }
}
