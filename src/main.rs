mod app;
mod data;
mod ui;
mod wasm;

use app::AppState;
use gpui::*;
use ui::root::RootWidget;

fn main() {
    let app = Application::new();
    app.run(move |cx| {
        gpui_component::init(cx);

        let mut state = AppState::new();

        if !state.config.openai_api_key.is_empty() {
            if let Err(e) = state.refresh() {
                eprintln!("Initial fetch error: {e}");
            }
        } else {
            state.data.error =
                Some("API key not set. Edit ~/.codex-widget/config.json".into());
        }

        let widget_data = state.data.clone();
        let _refresh_secs = state.config.refresh_interval_secs;

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: Size {
                        width: px(400.0),
                        height: px(600.0),
                    },
                })),
                titlebar: Some(TitlebarOptions {
                    title: Some("Codex Usage Widget".into()),
                    ..Default::default()
                }),
                is_movable: true,
                ..Default::default()
            },
            |window, cx| {
                let data_entity = cx.new(|_| widget_data);
                let view = cx.new(|cx| RootWidget::new(data_entity, cx));
                cx.new(|cx| gpui_component::Root::new(view, window, cx))
            },
        )
        .ok();
    });
}
