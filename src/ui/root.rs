use std::time::Duration;

use gpui::*;

use super::cost_card::CreditsCard;
use super::quota_bar::QuotaBar;
use super::theme::WidgetTheme;
use super::usage_panel::UsagePanel;
use crate::data::fetcher::CodexRpcClient;
use crate::data::models::{AppConfig, WidgetData};
use crate::platform;

const AUTO_REFRESH_SECS: u64 = 30;

/// Root widget: frameless compact window
pub struct RootWidget {
    pub data: Entity<WidgetData>,
    pub config: AppConfig,
    pub rpc: Option<CodexRpcClient>,
    pub show_settings: bool,
    pub opacity_initialized: bool,
    pub window_icon_initialized: bool,
}

impl RootWidget {
    pub fn new(
        data: Entity<WidgetData>,
        config: AppConfig,
        rpc: Option<CodexRpcClient>,
        cx: &mut Context<Self>,
    ) -> Self {
        // Start auto-refresh timer
        cx.spawn(async |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_secs(AUTO_REFRESH_SECS))
                    .await;
                let ok = this.update(cx, |this, cx| {
                    this.do_refresh(cx);
                });
                if ok.is_err() {
                    break;
                }
            }
        })
        .detach();

        Self {
            data,
            config,
            rpc,
            show_settings: false,
            opacity_initialized: false,
            window_icon_initialized: false,
        }
    }

    fn do_refresh(&mut self, cx: &mut Context<Self>) {
        let codex_path = self.config.codex_cli_path.clone();

        if let Some(rpc) = &mut self.rpc {
            let new_data = rpc.fetch_all();
            self.data.update(cx, |data, _cx| *data = new_data);
        } else {
            // Try reconnecting
            match CodexRpcClient::new(&codex_path) {
                Ok(mut client) => {
                    let new_data = client.fetch_all();
                    self.data.update(cx, |data, _cx| *data = new_data);
                    self.rpc = Some(client);
                }
                Err(_) => {}
            }
        }

        // Persist config (including window position tracked in render)
        self.save_config();
    }

    fn apply_opacity(&self, window: &Window) {
        platform::set_window_opacity(window, self.config.opacity);
    }

    fn save_config(&self) {
        self.config.save();
    }
}

impl Render for RootWidget {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Apply initial opacity on first render
        if !self.opacity_initialized {
            self.opacity_initialized = true;
            self.apply_opacity(window);
        }
        if !self.window_icon_initialized {
            self.window_icon_initialized = true;
            platform::initialize_window_icons(window);
        }

        // Track window position in memory (persisted to disk every auto-refresh)
        let bounds = window.bounds();
        self.config.window_x = Some(f32::from(bounds.origin.x));
        self.config.window_y = Some(f32::from(bounds.origin.y));

        let data = self.data.read(cx).clone();
        let last_updated = format_timestamp(data.last_updated);
        let plan_label = data
            .plan_type
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "—".into());

        let opacity_pct = (self.config.opacity * 100.0).round() as i32;

        let mut root = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(WidgetTheme::bg_primary())
            .text_color(WidgetTheme::text_primary())
            .rounded_lg()
            .border_1()
            .border_color(WidgetTheme::border())
            .overflow_hidden()
            // Top bar
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(WidgetTheme::border())
                    // Left side — drag area (content-width only, no flex_1 to avoid
                    // hitbox overlapping with buttons in WM_NCHITTEST)
                    .child(
                        div()
                            .id("titlebar-drag")
                            .flex()
                            .items_center()
                            .gap_2()
                            .window_control_area(WindowControlArea::Drag)
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(WidgetTheme::text_accent())
                                    .child("Codex 使用状況"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py(px(1.0))
                                    .rounded_md()
                                    .bg(WidgetTheme::bg_accent())
                                    .text_xs()
                                    .text_color(WidgetTheme::text_secondary())
                                    .child(plan_label),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            // Settings button
                            .child(
                                div()
                                    .id("btn-settings")
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(WidgetTheme::bg_accent())
                                    .text_xs()
                                    .text_color(WidgetTheme::text_secondary())
                                    .cursor_pointer()
                                    .hover(|s| s.bg(WidgetTheme::border()))
                                    .child(if self.show_settings { "✕" } else { "⚙" })
                                    .on_click(cx.listener(
                                        |this, _evt: &ClickEvent, _window, _cx| {
                                            this.show_settings = !this.show_settings;
                                        },
                                    )),
                            )
                            // Refresh button
                            .child(
                                div()
                                    .id("btn-refresh")
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(WidgetTheme::bg_accent())
                                    .text_xs()
                                    .text_color(WidgetTheme::text_secondary())
                                    .cursor_pointer()
                                    .hover(|s| s.bg(WidgetTheme::border()))
                                    .child("更新")
                                    .on_click(cx.listener(
                                        |this, _evt: &ClickEvent, _window, cx| {
                                            this.do_refresh(cx);
                                        },
                                    )),
                            )
                            // Close button
                            .child(
                                div()
                                    .id("btn-close")
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(WidgetTheme::bg_accent())
                                    .text_xs()
                                    .text_color(WidgetTheme::text_secondary())
                                    .cursor_pointer()
                                    .hover(|s| s.bg(WidgetTheme::danger()))
                                    .child("✕")
                                    .window_control_area(WindowControlArea::Close)
                                    .on_click(cx.listener(
                                        |this, _evt: &ClickEvent, window, cx| {
                                            if this.config.resident_in_tray {
                                                match platform::hide_window_to_tray(window) {
                                                    Ok(()) => {
                                                        this.save_config();
                                                        return;
                                                    }
                                                    Err(e) => {
                                                        this.data.update(cx, |data, _| {
                                                            data.error = Some(e)
                                                        });
                                                    }
                                                }
                                            }
                                            platform::remove_tray_icon();
                                            this.save_config();
                                            std::process::exit(0);
                                        },
                                    )),
                            ),
                    ),
            );

        // Settings panel
        if self.show_settings {
            root = root.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_3()
                    .bg(WidgetTheme::bg_secondary())
                    .border_b_1()
                    .border_color(WidgetTheme::border())
                    // Opacity control
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(WidgetTheme::text_primary())
                                    .child("透明度"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        div()
                                            .id("btn-opacity-down")
                                            .px_2()
                                            .py(px(2.0))
                                            .rounded_md()
                                            .bg(WidgetTheme::bg_accent())
                                            .text_xs()
                                            .text_color(WidgetTheme::text_secondary())
                                            .cursor_pointer()
                                            .hover(|s| s.bg(WidgetTheme::border()))
                                            .child("−")
                                            .on_click(cx.listener(
                                                |this, _evt: &ClickEvent, window, _cx| {
                                                    this.config.opacity = (this.config.opacity
                                                        - 0.05)
                                                        .clamp(0.1, 1.0);
                                                    this.apply_opacity(window);
                                                    this.save_config();
                                                },
                                            )),
                                    )
                                    .child(
                                        div()
                                            .w(px(48.0))
                                            .text_center()
                                            .text_sm()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(WidgetTheme::text_accent())
                                            .child(format!("{}%", opacity_pct)),
                                    )
                                    .child(
                                        div()
                                            .id("btn-opacity-up")
                                            .px_2()
                                            .py(px(2.0))
                                            .rounded_md()
                                            .bg(WidgetTheme::bg_accent())
                                            .text_xs()
                                            .text_color(WidgetTheme::text_secondary())
                                            .cursor_pointer()
                                            .hover(|s| s.bg(WidgetTheme::border()))
                                            .child("＋")
                                            .on_click(cx.listener(
                                                |this, _evt: &ClickEvent, window, _cx| {
                                                    this.config.opacity = (this.config.opacity
                                                        + 0.05)
                                                        .clamp(0.1, 1.0);
                                                    this.apply_opacity(window);
                                                    this.save_config();
                                                },
                                            )),
                                    ),
                            ),
                    )
                    // Opacity bar
                    .child(
                        div()
                            .w_full()
                            .h(px(6.0))
                            .bg(WidgetTheme::progress_bg())
                            .rounded(px(3.0))
                            .child(
                                div()
                                    .h_full()
                                    .rounded(px(3.0))
                                    .bg(WidgetTheme::text_accent())
                                    .w(relative(self.config.opacity)),
                            ),
                    )
                    // Always on top
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(WidgetTheme::text_primary())
                                    .child("常に最前面"),
                            )
                            .child(
                                div()
                                    .id("btn-always-on-top")
                                    .px_3()
                                    .py(px(2.0))
                                    .rounded_md()
                                    .bg(if self.config.always_on_top {
                                        WidgetTheme::success()
                                    } else {
                                        WidgetTheme::bg_accent()
                                    })
                                    .text_xs()
                                    .text_color(WidgetTheme::text_primary())
                                    .cursor_pointer()
                                    .hover(|s| s.bg(WidgetTheme::border()))
                                    .child(if self.config.always_on_top {
                                        "ON"
                                    } else {
                                        "OFF"
                                    })
                                    .on_click(cx.listener(
                                        |this, _evt: &ClickEvent, _window, _cx| {
                                            this.config.always_on_top = !this.config.always_on_top;
                                            this.save_config();
                                        },
                                    )),
                            ),
                    )
                    // Launch on startup
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(WidgetTheme::text_primary())
                                    .child("スタートアップ登録"),
                            )
                            .child(
                                div()
                                    .id("btn-startup")
                                    .px_3()
                                    .py(px(2.0))
                                    .rounded_md()
                                    .bg(if self.config.launch_on_startup {
                                        WidgetTheme::success()
                                    } else {
                                        WidgetTheme::bg_accent()
                                    })
                                    .text_xs()
                                    .text_color(WidgetTheme::text_primary())
                                    .cursor_pointer()
                                    .hover(|s| s.bg(WidgetTheme::border()))
                                    .child(if self.config.launch_on_startup {
                                        "ON"
                                    } else {
                                        "OFF"
                                    })
                                    .on_click(cx.listener(
                                        |this, _evt: &ClickEvent, _window, cx| {
                                            let next = !this.config.launch_on_startup;
                                            match platform::set_startup_enabled(next) {
                                                Ok(()) => {
                                                    this.config.launch_on_startup = next;
                                                    this.save_config();
                                                    this.data
                                                        .update(cx, |data, _| data.error = None);
                                                }
                                                Err(e) => {
                                                    this.data
                                                        .update(cx, |data, _| data.error = Some(e));
                                                }
                                            }
                                        },
                                    )),
                            ),
                    )
                    // Resident in tray mode
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(WidgetTheme::text_primary())
                                    .child("システムトレイ常駐"),
                            )
                            .child(
                                div()
                                    .id("btn-resident-in-tray")
                                    .px_3()
                                    .py(px(2.0))
                                    .rounded_md()
                                    .bg(if self.config.resident_in_tray {
                                        WidgetTheme::success()
                                    } else {
                                        WidgetTheme::bg_accent()
                                    })
                                    .text_xs()
                                    .text_color(WidgetTheme::text_primary())
                                    .cursor_pointer()
                                    .hover(|s| s.bg(WidgetTheme::border()))
                                    .child(if self.config.resident_in_tray {
                                        "ON"
                                    } else {
                                        "OFF"
                                    })
                                    .on_click(cx.listener(
                                        |this, _evt: &ClickEvent, _window, _cx| {
                                            this.config.resident_in_tray =
                                                !this.config.resident_in_tray;
                                            if !this.config.resident_in_tray {
                                                platform::remove_tray_icon();
                                            }
                                            this.save_config();
                                        },
                                    )),
                            ),
                    )
                    // Config path
                    .child(
                        div()
                            .text_color(WidgetTheme::text_secondary())
                            .text_xs()
                            .child(format!("設定: {}", AppConfig::config_path().display())),
                    )
                    // Auto-refresh info
                    .child(
                        div()
                            .text_color(WidgetTheme::text_secondary())
                            .text_xs()
                            .child(format!("自動更新: {}秒間隔", AUTO_REFRESH_SECS)),
                    ),
            );
        }

        // Content area
        let mut content = div()
            .id("content-scroll")
            .flex_1()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .overflow_y_scroll();

        if let Some(email) = &data.email {
            content = content.child(
                div()
                    .text_color(WidgetTheme::text_secondary())
                    .text_xs()
                    .child(format!("ログイン中: {email}")),
            );
        }

        if let Some(primary) = &data.primary_window {
            content = content.child(UsagePanel::new(primary.clone(), "5時間枠"));
        }

        if let Some(secondary) = &data.secondary_window {
            content = content.child(QuotaBar::new(secondary.clone()));
        }

        if let Some(credits) = &data.credits {
            content = content.child(CreditsCard::new(credits.clone()));
        }

        root = root.child(content);

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

        // Footer
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
                        .child(format!("更新: {last_updated}")),
                )
                .child(
                    div()
                        .text_color(WidgetTheme::text_secondary())
                        .text_xs()
                        .child("v0.2.5"),
                ),
        );

        root
    }
}

fn format_timestamp(ts: i64) -> String {
    if ts == 0 {
        return "未取得".to_string();
    }
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|d| d.format("%H:%M:%S").to_string())
        .unwrap_or_else(|| "不明".to_string())
}
