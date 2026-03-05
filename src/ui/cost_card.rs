use gpui::*;

use crate::data::models::{CostSummary, ModelUsage};
use super::theme::WidgetTheme;

/// Cost display card with today/week/month breakdown and model usage list
#[derive(IntoElement)]
pub struct CostCard {
    pub costs: CostSummary,
    pub models: Vec<ModelUsage>,
}

impl CostCard {
    pub fn new(costs: CostSummary, models: Vec<ModelUsage>) -> Self {
        Self { costs, models }
    }
}

impl RenderOnce for CostCard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut root = div()
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
                    .child("Costs"),
            )
            // 3-column layout: Today / This Week / This Month
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(cost_cell("Today", self.costs.today_usd))
                    .child(cost_cell("This Week", self.costs.week_usd))
                    .child(cost_cell("This Month", self.costs.month_usd)),
            )
            // Daily average + forecast
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .text_color(WidgetTheme::text_secondary())
                            .text_xs()
                            .child(format!("Daily avg: ${:.2}", self.costs.daily_average_usd)),
                    )
                    .child(
                        div()
                            .text_color(WidgetTheme::text_secondary())
                            .text_xs()
                            .child(format!("Forecast: ${:.2}/mo", self.costs.forecast_month_usd)),
                    ),
            );

        // Model usage list
        if !self.models.is_empty() {
            root = root.child(
                div()
                    .mt_1()
                    .pt_2()
                    .border_t_1()
                    .border_color(WidgetTheme::border())
                    .child(
                        div()
                            .text_color(WidgetTheme::text_secondary())
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .mb_1()
                            .child("By Model"),
                    ),
            );
            for model in &self.models {
                let tokens_str = format_tokens_short(model.input_tokens + model.output_tokens);
                root = root.child(
                    div()
                        .flex()
                        .justify_between()
                        .px_1()
                        .child(
                            div()
                                .text_color(WidgetTheme::text_secondary())
                                .text_xs()
                                .child(model.model.clone()),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    div()
                                        .text_color(WidgetTheme::text_secondary())
                                        .text_xs()
                                        .child(tokens_str),
                                )
                                .child(
                                    div()
                                        .text_color(WidgetTheme::text_accent())
                                        .text_xs()
                                        .child(format!("${:.2}", model.cost_usd)),
                                ),
                        ),
                );
            }
        }

        root
    }
}

/// Renders a single cost cell (label + dollar amount)
fn cost_cell(label: &str, amount: f64) -> Div {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .p_2()
        .bg(WidgetTheme::bg_accent())
        .rounded_md()
        .child(
            div()
                .text_color(WidgetTheme::text_secondary())
                .text_xs()
                .child(label.to_string()),
        )
        .child(
            div()
                .text_color(WidgetTheme::text_primary())
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .child(format!("${:.2}", amount)),
        )
}

fn format_tokens_short(n: u64) -> String {
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
