use anyhow::Result;
use chrono::{Datelike, Utc};

use crate::data::fetcher::DataFetcher;
use crate::data::models::{AppConfig, WidgetData};
use crate::wasm::bridge::{self, WasmBridge};

/// Top-level application state
pub struct AppState {
    pub config: AppConfig,
    pub data: WidgetData,
    pub wasm: Option<WasmBridge>,
    pub fetcher: DataFetcher,
    pub is_loading: bool,
}

impl AppState {
    pub fn new() -> Self {
        let config = Self::load_config();
        let fetcher = DataFetcher::new(&config);
        let wasm = WasmBridge::new().ok(); // Failures are non-fatal

        Self {
            config,
            data: WidgetData::default(),
            wasm,
            fetcher,
            is_loading: false,
        }
    }

    /// Load config from ~/.codex-widget/config.json, creating defaults if absent
    pub fn load_config() -> AppConfig {
        let config_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".codex-widget");

        let config_path = config_dir.join("config.json");

        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => match serde_json::from_str::<AppConfig>(&content) {
                    Ok(cfg) => return cfg,
                    Err(e) => eprintln!("Config parse error: {e}"),
                },
                Err(e) => eprintln!("Config read error: {e}"),
            }
        } else {
            // Create default config file
            let _ = std::fs::create_dir_all(&config_dir);
            let default = AppConfig::default();
            if let Ok(json) = serde_json::to_string_pretty(&default) {
                let _ = std::fs::write(&config_path, json);
            }
            eprintln!(
                "Created default config at {}. Please set your API key.",
                config_path.display()
            );
        }

        AppConfig::default()
    }

    /// Refresh data from API and apply calculations
    pub fn refresh(&mut self) -> Result<()> {
        self.is_loading = true;
        let result = self.fetcher.fetch_all();
        self.is_loading = false;

        match result {
            Ok(mut widget_data) => {
                self.apply_config_limits(&mut widget_data);
                self.apply_calculations(&mut widget_data);
                self.data = widget_data;
                Ok(())
            }
            Err(e) => {
                self.data.error = Some(format!("Refresh failed: {e}"));
                Err(e)
            }
        }
    }

    /// Apply token limits from config to widget data
    fn apply_config_limits(&self, data: &mut WidgetData) {
        data.session.tokens_limit = self.config.session_token_limit;
        data.weekly.tokens_limit = self.config.weekly_token_limit;
    }

    /// Run WASM (or fallback) calculations
    fn apply_calculations(&mut self, data: &mut WidgetData) {
        let now = Utc::now().timestamp();

        // Usage percentages
        data.session.usage_pct = match &mut self.wasm {
            Some(w) => w.calculate_usage_pct(data.session.tokens_used, data.session.tokens_limit),
            None => bridge::fallback_usage_pct(data.session.tokens_used, data.session.tokens_limit),
        };

        data.weekly.usage_pct = match &mut self.wasm {
            Some(w) => w.calculate_usage_pct(data.weekly.tokens_used, data.weekly.tokens_limit),
            None => bridge::fallback_usage_pct(data.weekly.tokens_used, data.weekly.tokens_limit),
        };

        // Time remaining (weekly reset = next Monday 00:00 UTC)
        let weekly_reset = next_monday_ts(now);
        data.weekly.reset_at = weekly_reset;
        data.session.reset_at = weekly_reset;

        let remaining_secs = match &mut self.wasm {
            Some(w) => w.calculate_time_remaining(weekly_reset, now),
            None => bridge::fallback_time_remaining(weekly_reset, now),
        };
        data.weekly.days_remaining = (remaining_secs / 86400) as u32;
    }
}

/// Calculate timestamp of next Monday 00:00 UTC
fn next_monday_ts(now: i64) -> i64 {
    let dt = chrono::DateTime::from_timestamp(now, 0).unwrap_or_default();
    let weekday = dt.weekday().num_days_from_monday(); // Mon=0, Sun=6
    let days_until_monday = if weekday == 0 { 7 } else { 7 - weekday };
    let next_mon = dt.date_naive() + chrono::Duration::days(days_until_monday as i64);
    next_mon
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp()
}
