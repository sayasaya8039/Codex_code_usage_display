use serde::{Deserialize, Serialize};

// ── Codex CLI app-server JSON-RPC protocol models ──

/// JSON-RPC request envelope
#[derive(Debug, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: &'static str,
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

/// JSON-RPC response envelope
#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse {
    pub id: Option<u64>,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

// ── Account models ──

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountResponse {
    pub account: Option<Account>,
    pub requires_openai_auth: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum Account {
    #[serde(rename = "chatgpt")]
    ChatGpt {
        email: String,
        #[serde(rename = "planType")]
        plan_type: PlanType,
    },
    #[serde(rename = "apiKey")]
    ApiKey,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PlanType {
    Free,
    Go,
    Plus,
    Pro,
    Team,
    Business,
    Enterprise,
    Edu,
    Unknown,
}

impl std::fmt::Display for PlanType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanType::Free => write!(f, "Free"),
            PlanType::Go => write!(f, "Go"),
            PlanType::Plus => write!(f, "Plus"),
            PlanType::Pro => write!(f, "Pro"),
            PlanType::Team => write!(f, "Team"),
            PlanType::Business => write!(f, "Business"),
            PlanType::Enterprise => write!(f, "Enterprise"),
            PlanType::Edu => write!(f, "Edu"),
            PlanType::Unknown => write!(f, "Unknown"),
        }
    }
}

// ── Rate limit models ──

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAccountRateLimitsResponse {
    pub rate_limits: RateLimitSnapshot,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitSnapshot {
    pub primary: Option<RateLimitWindow>,
    pub secondary: Option<RateLimitWindow>,
    pub credits: Option<CreditsSnapshot>,
    #[serde(rename = "planType")]
    pub plan_type: Option<PlanType>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimitWindow {
    pub used_percent: i32,
    pub resets_at: Option<i64>,
    pub window_duration_mins: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditsSnapshot {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<String>,
}

// ── Widget display model ──

#[derive(Debug, Clone, Default)]
pub struct WidgetData {
    pub email: Option<String>,
    pub plan_type: Option<PlanType>,
    pub primary_window: Option<WindowDisplay>,
    pub secondary_window: Option<WindowDisplay>,
    pub credits: Option<CreditsDisplay>,
    pub last_updated: i64,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WindowDisplay {
    pub label: String,
    pub used_percent: i32,
    pub remaining_percent: i32,
    pub resets_at: Option<i64>,
    pub window_duration_mins: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct CreditsDisplay {
    pub has_credits: bool,
    pub unlimited: bool,
    pub balance: Option<String>,
}

impl WidgetData {
    pub fn from_rpc(
        account_resp: Option<&GetAccountResponse>,
        limits_resp: Option<&GetAccountRateLimitsResponse>,
    ) -> Self {
        let mut data = WidgetData {
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64,
            ..Default::default()
        };

        if let Some(acct) = account_resp {
            if let Some(Account::ChatGpt { email, plan_type }) = &acct.account {
                data.email = Some(email.clone());
                data.plan_type = Some(plan_type.clone());
            }
        }

        if let Some(limits) = limits_resp {
            let snap = &limits.rate_limits;

            if data.plan_type.is_none() {
                data.plan_type = snap.plan_type.clone();
            }

            if let Some(p) = &snap.primary {
                data.primary_window = Some(WindowDisplay {
                    label: format_window_label(p.window_duration_mins),
                    used_percent: p.used_percent,
                    remaining_percent: 100 - p.used_percent.min(100),
                    resets_at: p.resets_at,
                    window_duration_mins: p.window_duration_mins,
                });
            }

            if let Some(s) = &snap.secondary {
                data.secondary_window = Some(WindowDisplay {
                    label: format_window_label(s.window_duration_mins),
                    used_percent: s.used_percent,
                    remaining_percent: 100 - s.used_percent.min(100),
                    resets_at: s.resets_at,
                    window_duration_mins: s.window_duration_mins,
                });
            }

            if let Some(c) = &snap.credits {
                data.credits = Some(CreditsDisplay {
                    has_credits: c.has_credits,
                    unlimited: c.unlimited,
                    balance: c.balance.clone(),
                });
            }
        }

        data
    }
}

fn format_window_label(duration_mins: Option<i64>) -> String {
    match duration_mins {
        Some(m) if m >= 1440 => format!("{}日間", m / 1440),
        Some(m) if m >= 60 => format!("{}時間枠", m / 60),
        Some(m) => format!("{}分枠", m),
        None => "使用枠".to_string(),
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_codex_path")]
    pub codex_cli_path: String,
    #[serde(default = "default_refresh_secs")]
    pub refresh_interval_secs: u64,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default)]
    pub window_x: Option<f32>,
    #[serde(default)]
    pub window_y: Option<f32>,
    #[serde(default)]
    pub launch_on_startup: bool,
    #[serde(default)]
    pub resident_in_tray: bool,
}

fn default_codex_path() -> String {
    "codex".to_string()
}
fn default_refresh_secs() -> u64 {
    60
}
fn default_opacity() -> f32 {
    0.95
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            codex_cli_path: default_codex_path(),
            refresh_interval_secs: default_refresh_secs(),
            always_on_top: true,
            opacity: default_opacity(),
            window_x: None,
            window_y: None,
            launch_on_startup: false,
            resident_in_tray: false,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".codex-widget")
            .join("config.json")
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }
}
