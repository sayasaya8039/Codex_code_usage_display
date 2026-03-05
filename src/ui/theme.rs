use gpui::{Rgba, rgb};

/// Dark theme color palette for the widget
pub struct WidgetTheme;

impl WidgetTheme {
    // Backgrounds
    pub fn bg_primary() -> Rgba { rgb(0x1a1a2e) }
    pub fn bg_secondary() -> Rgba { rgb(0x16213e) }
    pub fn bg_accent() -> Rgba { rgb(0x0f3460) }

    // Text
    pub fn text_primary() -> Rgba { rgb(0xe8e8e8) }
    pub fn text_secondary() -> Rgba { rgb(0xa0a0b0) }
    pub fn text_accent() -> Rgba { rgb(0x00d2ff) }

    // Status
    pub fn success() -> Rgba { rgb(0x00c853) }
    pub fn warning() -> Rgba { rgb(0xff9100) }
    pub fn danger() -> Rgba { rgb(0xff1744) }

    // Progress bar
    pub fn progress_bg() -> Rgba { rgb(0x2a2a3e) }
    pub fn progress_fill() -> Rgba { rgb(0x536dfe) }

    // Border
    pub fn border() -> Rgba { rgb(0x2a2a4a) }

    /// Returns the appropriate progress bar color based on usage percentage
    pub fn usage_color(pct: f64) -> Rgba {
        if pct >= 90.0 {
            Self::danger()
        } else if pct >= 70.0 {
            Self::warning()
        } else {
            Self::progress_fill()
        }
    }
}
