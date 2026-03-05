use gpui::*;

use crate::data::models::SessionUsage;
use super::countdown::Countdown;
use super::theme::WidgetTheme;

/// Session usage panel showing token usage, requests, and progress
#[derive(IntoElement)]
pub struct UsagePanel {
    pub session: SessionUsage,
}

impl UsagePanel {
    pub fn new(session: SessionUsage) -> Self {
        Self { session }
    }

    fn format_tokens(n: u64) -> String {
        if n >= 1_000_000_000 {
            format!("{:.1}B", n as f64 / 1_000_000_000.0)
        } else if n >= 1_000_000 {
            format!("{:.1}M", n as f64 / 1_000_000.0)
        } else if n >= 1_000 {
            format!("{:.1}K", n as f64 / 1_000.0)
        } else {
            n.to_string()
        }
    }
}

impl RenderOnce for UsagePanel {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let used_str = Self::format_tokens(self.session.tokens_used);
        let limit_str = Self::format_tokens(self.session.tokens_limit);
        let pct = self.session.usage_pct;
        let fill_color = WidgetTheme::usage_color(pct);

        // Clamp bar width to 0..100
        let bar_pct = pct.clamp(0.0, 100.0);

        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .bg(WidgetTheme::bg_secondary())
            .rounded_lg()
            .border_1()
            .border_color(WidgetTheme::border())
            // Header
            .child(
                div()
                    .text_color(WidgetTheme::text_primary())
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .child("Session Usage"),
            )
            // Token usage text
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .text_color(WidgetTheme::text_accent())
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .child(format!("{} / {} tokens", used_str, limit_str)),
                    )
                    .child(
                        div()
                            .text_color(WidgetTheme::text_secondary())
                            .text_sm()
                            .child(format!("{:.1}%", pct)),
                    ),
            )
            // Progress bar
            .child(
                div()
                    .w_full()
                    .h(px(8.0))
                    .bg(WidgetTheme::progress_bg())
                    .rounded(px(4.0))
                    .child(
                        div()
                            .h_full()
                            .rounded(px(4.0))
                            .bg(fill_color)
                            .w(relative(bar_pct as f32 / 100.0)),
                    ),
            )
            // Requests + Countdown row
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_color(WidgetTheme::text_secondary())
                            .text_xs()
                            .child(format!("{} requests", self.session.requests_made)),
                    )
                    .child(Countdown::new(self.session.reset_at)),
            )
    }
}
