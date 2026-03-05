use gpui::*;

use crate::data::models::WeeklyQuota;
use super::theme::WidgetTheme;

/// Weekly quota progress bar with remaining token count and reset info
#[derive(IntoElement)]
pub struct QuotaBar {
    pub quota: WeeklyQuota,
}

impl QuotaBar {
    pub fn new(quota: WeeklyQuota) -> Self {
        Self { quota }
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

impl RenderOnce for QuotaBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let pct = self.quota.usage_pct.clamp(0.0, 100.0);
        let fill_color = WidgetTheme::usage_color(self.quota.usage_pct);
        let remaining = self.quota.tokens_limit.saturating_sub(self.quota.tokens_used);
        let remaining_str = Self::format_tokens(remaining);

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
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .text_color(WidgetTheme::text_primary())
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .child("Weekly Quota"),
                    )
                    .child(
                        div()
                            .text_color(WidgetTheme::text_secondary())
                            .text_sm()
                            .child(format!("{:.1}%", self.quota.usage_pct)),
                    ),
            )
            // Large progress bar
            .child(
                div()
                    .w_full()
                    .h(px(14.0))
                    .bg(WidgetTheme::progress_bg())
                    .rounded(px(7.0))
                    .child(
                        div()
                            .h_full()
                            .rounded(px(7.0))
                            .bg(fill_color)
                            .w(relative(pct as f32 / 100.0)),
                    ),
            )
            // Remaining + reset info
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .text_color(WidgetTheme::text_secondary())
                            .text_xs()
                            .child(format!("{} remaining", remaining_str)),
                    )
                    .child(
                        div()
                            .text_color(WidgetTheme::text_secondary())
                            .text_xs()
                            .child(format!(
                                "Resets in {} day{}",
                                self.quota.days_remaining,
                                if self.quota.days_remaining != 1 { "s" } else { "" }
                            )),
                    ),
            )
    }
}
