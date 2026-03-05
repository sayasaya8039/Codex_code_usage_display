use anyhow::{Context, Result};
use chrono::Utc;

use super::api::{self, ApiParams};
use super::models::*;

/// Synchronous HTTP client for OpenAI Platform API
pub struct DataFetcher {
    api_key: String,
    org_id: String,
}

impl DataFetcher {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            api_key: config.openai_api_key.clone(),
            org_id: config.organization_id.clone(),
        }
    }

    /// Fetch token usage for the past 7 days, grouped by model
    pub fn fetch_usage(&self) -> Result<UsageResponse> {
        let now = Utc::now().timestamp();
        let seven_days_ago = now - 7 * 86400;

        let params = ApiParams {
            start_time: seven_days_ago,
            end_time: now,
            bucket_width: "1d".into(),
            group_by: vec!["model".into(), "project_id".into()],
        };

        let url = api::usage_completions_url();
        let mut req = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key));

        if !self.org_id.is_empty() {
            req = req.set("OpenAI-Organization", &self.org_id);
        }

        for (key, val) in params.to_query_pairs() {
            req = req.query(key, &val);
        }

        let resp: UsageResponse = req
            .call()
            .context("Failed to call usage API")?
            .into_json()
            .context("Failed to parse usage response")?;

        Ok(resp)
    }

    /// Fetch cost data for the past 30 days
    pub fn fetch_costs(&self) -> Result<CostResponse> {
        let now = Utc::now().timestamp();
        let thirty_days_ago = now - 30 * 86400;

        let params = ApiParams {
            start_time: thirty_days_ago,
            end_time: now,
            bucket_width: "1d".into(),
            group_by: vec![],
        };

        let url = api::usage_costs_url();
        let mut req = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_key));

        if !self.org_id.is_empty() {
            req = req.set("OpenAI-Organization", &self.org_id);
        }

        for (key, val) in params.to_query_pairs() {
            req = req.query(key, &val);
        }

        let resp: CostResponse = req
            .call()
            .context("Failed to call costs API")?
            .into_json()
            .context("Failed to parse costs response")?;

        Ok(resp)
    }

    /// Fetch both usage and costs, aggregate into WidgetData
    pub fn fetch_all(&self) -> Result<WidgetData> {
        let now = Utc::now().timestamp();

        let usage = self.fetch_usage();
        let costs = self.fetch_costs();

        let mut widget = WidgetData {
            last_updated: now,
            ..Default::default()
        };

        // Process usage data
        match usage {
            Ok(usage_resp) => {
                self.aggregate_usage(&usage_resp, &mut widget);
            }
            Err(e) => {
                widget.error = Some(format!("Usage API error: {e}"));
            }
        }

        // Process cost data
        match costs {
            Ok(cost_resp) => {
                self.aggregate_costs(&cost_resp, &mut widget);
            }
            Err(e) => {
                let msg = format!("Costs API error: {e}");
                widget.error = Some(match widget.error.take() {
                    Some(prev) => format!("{prev}; {msg}"),
                    None => msg,
                });
            }
        }

        Ok(widget)
    }

    /// Aggregate usage buckets into WidgetData fields
    fn aggregate_usage(&self, resp: &UsageResponse, widget: &mut WidgetData) {
        let mut total_input: u64 = 0;
        let mut total_output: u64 = 0;
        let mut total_requests: u64 = 0;

        // Per-model aggregation
        let mut model_map: std::collections::HashMap<String, ModelUsage> =
            std::collections::HashMap::new();

        for bucket in &resp.data {
            for result in &bucket.results {
                total_input += result.input_tokens;
                total_output += result.output_tokens;
                total_requests += result.num_model_requests;

                let model_name = result
                    .model
                    .clone()
                    .unwrap_or_else(|| "unknown".into());

                let entry = model_map
                    .entry(model_name.clone())
                    .or_insert_with(|| ModelUsage {
                        model: model_name,
                        ..Default::default()
                    });
                entry.input_tokens += result.input_tokens;
                entry.output_tokens += result.output_tokens;
                entry.requests += result.num_model_requests;
            }
        }

        let total_tokens = total_input + total_output;

        widget.weekly = WeeklyQuota {
            tokens_used: total_tokens,
            tokens_limit: 0, // Set from config by AppState
            usage_pct: 0.0,  // Calculated by AppState with WASM or fallback
            reset_at: 0,
            days_remaining: 0,
        };

        widget.session = SessionUsage {
            tokens_used: total_tokens,
            tokens_limit: 0, // Set from config by AppState
            requests_made: total_requests,
            usage_pct: 0.0,
            reset_at: 0,
        };

        widget.models = model_map.into_values().collect();
        // Sort by total tokens descending
        widget
            .models
            .sort_by(|a, b| {
                (b.input_tokens + b.output_tokens).cmp(&(a.input_tokens + a.output_tokens))
            });
    }

    /// Aggregate cost buckets into CostSummary
    fn aggregate_costs(&self, resp: &CostResponse, widget: &mut WidgetData) {
        let now = Utc::now();
        let today_start = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let week_start = today_start - 7 * 86400;

        let mut today_total = 0.0_f64;
        let mut week_total = 0.0_f64;
        let mut month_total = 0.0_f64;
        let mut daily_costs: Vec<DailyCost> = Vec::new();
        let mut num_days: u32 = 0;

        for bucket in &resp.data {
            let day_amount: f64 = bucket.results.iter().map(|r| r.amount.value).sum();
            month_total += day_amount;

            let date_str = chrono::DateTime::from_timestamp(bucket.start_time, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_default();

            daily_costs.push(DailyCost {
                date: date_str,
                amount_usd: day_amount,
            });

            num_days += 1;

            if bucket.start_time >= today_start {
                today_total += day_amount;
            }
            if bucket.start_time >= week_start {
                week_total += day_amount;
            }
        }

        let daily_average = if num_days > 0 {
            month_total / num_days as f64
        } else {
            0.0
        };

        // Forecast: daily_average * days_in_current_month
        let days_in_month = days_in_current_month(now);
        let forecast = daily_average * days_in_month as f64;

        widget.costs = CostSummary {
            today_usd: today_total,
            week_usd: week_total,
            month_usd: month_total,
            daily_average_usd: daily_average,
            forecast_month_usd: forecast,
            daily_costs,
        };
    }
}

/// Return the number of days in the current month
fn days_in_current_month(now: chrono::DateTime<Utc>) -> u32 {
    let year = now.format("%Y").to_string().parse::<i32>().unwrap_or(2026);
    let month = now.format("%m").to_string().parse::<u32>().unwrap_or(1);
    // Calculate last day of this month
    if month == 12 {
        let next = chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1);
        let this = chrono::NaiveDate::from_ymd_opt(year, month, 1);
        match (next, this) {
            (Some(n), Some(t)) => (n - t).num_days() as u32,
            _ => 30,
        }
    } else {
        let next = chrono::NaiveDate::from_ymd_opt(year, month + 1, 1);
        let this = chrono::NaiveDate::from_ymd_opt(year, month, 1);
        match (next, this) {
            (Some(n), Some(t)) => (n - t).num_days() as u32,
            _ => 30,
        }
    }
}
