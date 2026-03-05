use gpui::*;

use crate::data::models::WidgetData;
use super::cost_card::CostCard;
use super::quota_bar::QuotaBar;
use super::theme::WidgetTheme;
use super::usage_panel::UsagePanel;

/// Root widget: 400x600px compact window
pub struct RootWidget {
    pub data: Entity<WidgetData>,
}

impl RootWidget {
    pub fn new(data: Entity<WidgetData>, _cx: &mut Context<Self>) -> Self {
        Self { data }
    }
}

impl Render for RootWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let data = self.data.read(cx).clone();

        let last_updated = format_timestamp(data.last_updated);

        let mut root = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(WidgetTheme::bg_primary())
            .text_color(WidgetTheme::text_primary())
            // Top bar: title + refresh
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(WidgetTheme::border())
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(WidgetTheme::text_accent())
                            .child("Codex Usage"),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(WidgetTheme::bg_accent())
                            .text_xs()
                            .text_color(WidgetTheme::text_secondary())
                            .cursor_pointer()
                            .child("Refresh"),
                    ),
            )
            // Content area: scrollable vertical stack
            .child(
                div()
                    .id("content-scroll")
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_3()
                    .overflow_y_scroll()
                    // Usage panel
                    .child(UsagePanel::new(data.session))
                    // Quota bar
                    .child(QuotaBar::new(data.weekly))
                    // Cost card
                    .child(CostCard::new(data.costs, data.models)),
            );

        // Error banner (if any)
        if let Some(err) = &data.error {
            root = root.child(
                div()
                    .mx_3()
                    .mb_1()
                    .p_2()
                    .bg(WidgetTheme::danger())
                    .rounded_md()
                    .text_xs()
                    .text_color(WidgetTheme::text_primary())
                    .child(err.clone()),
            );
        }

        // Footer: last updated + version
        root = root.child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .px_3()
                .py_1()
                .border_t_1()
                .border_color(WidgetTheme::border())
                .child(
                    div()
                        .text_color(WidgetTheme::text_secondary())
                        .text_xs()
                        .child(format!("Updated: {}", last_updated)),
                )
                .child(
                    div()
                        .text_color(WidgetTheme::text_secondary())
                        .text_xs()
                        .child("v0.1.1"),
                ),
        );

        root
    }
}

fn format_timestamp(ts: i64) -> String {
    if ts == 0 {
        return "Never".to_string();
    }
    let dt = chrono::DateTime::from_timestamp(ts, 0);
    match dt {
        Some(d) => d.format("%H:%M:%S").to_string(),
        None => "Unknown".to_string(),
    }
}
