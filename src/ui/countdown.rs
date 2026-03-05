use gpui::*;

use super::theme::WidgetTheme;

/// Displays a countdown timer in "Xh Ym Zs" format
pub struct Countdown {
    /// Target timestamp (Unix epoch seconds)
    pub reset_at: i64,
}

impl Countdown {
    pub fn new(reset_at: i64) -> Self {
        Self { reset_at }
    }

    fn remaining_seconds(&self) -> i64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        (self.reset_at - now).max(0)
    }

    fn format_remaining(&self) -> String {
        let secs = self.remaining_seconds();
        if secs <= 0 {
            return "Expired".to_string();
        }
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        if h > 0 {
            format!("{}h {}m {}s", h, m, s)
        } else if m > 0 {
            format!("{}m {}s", m, s)
        } else {
            format!("{}s", s)
        }
    }

    fn is_urgent(&self) -> bool {
        self.remaining_seconds() < 3600
    }
}

impl IntoElement for Countdown {
    type Element = <Div as IntoElement>::Element;

    fn into_element(self) -> Self::Element {
        let text = self.format_remaining();
        let color = if self.is_urgent() {
            WidgetTheme::danger()
        } else {
            WidgetTheme::text_accent()
        };

        div()
            .flex()
            .items_center()
            .gap_1()
            .child(
                div()
                    .text_color(WidgetTheme::text_secondary())
                    .text_sm()
                    .child("Resets in "),
            )
            .child(
                div()
                    .text_color(color)
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .child(text),
            )
            .into_element()
    }
}
