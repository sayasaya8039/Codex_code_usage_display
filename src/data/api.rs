/// OpenAI Platform API endpoint definitions
pub const BASE_URL: &str = "https://api.openai.com/v1";

/// Usage completions endpoint (token usage by model/project)
pub fn usage_completions_url() -> String {
    format!("{}/organization/usage/completions", BASE_URL)
}

/// Cost reporting endpoint
pub fn usage_costs_url() -> String {
    format!("{}/organization/costs", BASE_URL)
}

/// Query parameters for usage/cost endpoints
pub struct ApiParams {
    pub start_time: i64,
    pub end_time: i64,
    pub bucket_width: String,
    pub group_by: Vec<String>,
}

impl ApiParams {
    /// Build query string for ureq
    pub fn to_query_pairs(&self) -> Vec<(&str, String)> {
        let mut pairs = vec![
            ("start_time", self.start_time.to_string()),
            ("end_time", self.end_time.to_string()),
            ("bucket_width", self.bucket_width.clone()),
        ];
        for g in &self.group_by {
            pairs.push(("group_by[]", g.clone()));
        }
        pairs
    }
}
