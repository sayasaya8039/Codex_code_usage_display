use serde::{Deserialize, Serialize};

/// OpenAI Usage API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageResponse {
    pub object: String,
    pub data: Vec<UsageBucket>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub next_page: Option<String>,
}

/// A single time bucket of usage data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageBucket {
    pub start_time: i64,
    pub end_time: i64,
    pub results: Vec<UsageResult>,
}

/// Usage result within a bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageResult {
    pub object: String,
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub input_cached_tokens: u64,
    #[serde(default)]
    pub num_model_requests: u64,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

/// Cost data from OpenAI API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostResponse {
    pub object: String,
    pub data: Vec<CostBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBucket {
    pub start_time: i64,
    pub end_time: i64,
    pub results: Vec<CostResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostResult {
    pub object: String,
    #[serde(default)]
    pub amount: CostAmount,
    #[serde(default)]
    pub line_item: Option<String>,
    #[serde(default)]
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostAmount {
    #[serde(default)]
    pub value: f64,
    #[serde(default)]
    pub currency: String,
}

// ── Widget display models ──

/// Aggregated data ready for widget display
#[derive(Debug, Clone, Default)]
pub struct WidgetData {

    pub session: SessionUsage,
    pub weekly: WeeklyQuota,
    pub costs: CostSummary,
    pub models: Vec<ModelUsage>,
    pub last_updated: i64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SessionUsage {
    pub tokens_used: u64,
    pub tokens_limit: u64,
    pub requests_made: u64,
    pub usage_pct: f64,
    pub reset_at: i64,
}

#[derive(Debug, Clone, Default)]
pub struct WeeklyQuota {
    pub tokens_used: u64,
    pub tokens_limit: u64,
    pub usage_pct: f64,
    pub reset_at: i64,
    pub days_remaining: u32,
}

#[derive(Debug, Clone, Default)]
pub struct CostSummary {
    pub today_usd: f64,
    pub week_usd: f64,
    pub month_usd: f64,
    pub daily_average_usd: f64,
    pub forecast_month_usd: f64,
    pub daily_costs: Vec<DailyCost>,
}

#[derive(Debug, Clone, Default)]
pub struct DailyCost {
    pub date: String,
    pub amount_usd: f64,
}

#[derive(Debug, Clone, Default)]
pub struct ModelUsage {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub requests: u64,
    pub cost_usd: f64,
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub openai_api_key: String,
    #[serde(default = "default_org_id")]
    pub organization_id: String,
    #[serde(default = "default_refresh_secs")]
    pub refresh_interval_secs: u64,
    #[serde(default = "default_session_limit")]
    pub session_token_limit: u64,
    #[serde(default = "default_weekly_limit")]
    pub weekly_token_limit: u64,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
}

fn default_org_id() -> String { String::new() }
fn default_refresh_secs() -> u64 { 300 }
fn default_session_limit() -> u64 { 500_000_000 }
fn default_weekly_limit() -> u64 { 5_000_000_000 }
fn default_opacity() -> f32 { 0.95 }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            openai_api_key: String::new(),
            organization_id: default_org_id(),
            refresh_interval_secs: default_refresh_secs(),
            session_token_limit: default_session_limit(),
            weekly_token_limit: default_weekly_limit(),
            always_on_top: true,
            opacity: default_opacity(),
        }
    }
}
