use anyhow::{Context, Result};
use wasmtime::{Engine, Instance, Memory, Module, Store};

/// Buffer size for WASM string formatting functions
const WASM_BUF_SIZE: u32 = 128;
/// Offset in WASM linear memory reserved for the format buffer
const WASM_BUF_OFFSET: u32 = 1024;

/// Bridge to Zig-compiled WASM data-processing module.
/// Falls back to pure-Rust implementations if WASM is unavailable.
pub struct WasmBridge {
    store: Store<()>,
    instance: Instance,
    memory: Memory,
}

impl WasmBridge {
    /// Load the WASM module. Searches:
    /// 1. Same directory as the executable: codex_data_processor.wasm
    /// 2. Fallback: zig-wasm/zig-out/lib/codex_data_processor.wasm (dev builds)
    pub fn new() -> Result<Self> {
        let wasm_path = Self::find_wasm_file()?;

        let engine = Engine::default();
        let module = Module::from_file(&engine, &wasm_path)
            .with_context(|| format!("Failed to load WASM module: {}", wasm_path.display()))?;

        let mut store = Store::new(&engine, ());
        let instance = Instance::new(&mut store, &module, &[])
            .context("Failed to instantiate WASM module")?;

        let memory = instance
            .get_memory(&mut store, "memory")
            .context("WASM module has no exported 'memory'")?;

        Ok(Self {
            store,
            instance,
            memory,
        })
    }

    fn find_wasm_file() -> Result<std::path::PathBuf> {
        // 1. Next to executable
        if let Ok(exe) = std::env::current_exe() {
            let dir = exe.parent().unwrap_or(std::path::Path::new("."));
            let candidate = dir.join("codex_data_processor.wasm");
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        // 2. Dev fallback
        let dev = std::path::PathBuf::from("zig-wasm/zig-out/lib/codex_data_processor.wasm");
        if dev.exists() {
            return Ok(dev);
        }

        anyhow::bail!("codex_data_processor.wasm not found")
    }

    // ── Exported WASM functions ──
    // Zig export names: calculate_usage_percentage, calculate_time_remaining,
    //   calculate_daily_average, calculate_forecast, calculate_cost_forecast,
    //   format_tokens, format_cost_usd, format_duration, format_percentage

    /// Calculate usage percentage: (used / total) * 100
    pub fn calculate_usage_pct(&mut self, used: u64, total: u64) -> f64 {
        self.call_f64(
            "calculate_usage_percentage",
            &[wasmtime::Val::I64(used as i64), wasmtime::Val::I64(total as i64)],
        )
        .unwrap_or_else(|_| fallback_usage_pct(used, total))
    }

    /// Calculate seconds remaining until reset
    pub fn calculate_time_remaining(&mut self, reset_ts: i64, now_ts: i64) -> i64 {
        self.call_i64(
            "calculate_time_remaining",
            &[wasmtime::Val::I64(reset_ts), wasmtime::Val::I64(now_ts)],
        )
        .unwrap_or_else(|_| fallback_time_remaining(reset_ts, now_ts))
    }

    /// Calculate daily average cost in cents
    pub fn calculate_daily_avg(&mut self, total_cents: u64, days: u32) -> u64 {
        self.call_u64(
            "calculate_daily_average",
            &[wasmtime::Val::I64(total_cents as i64), wasmtime::Val::I32(days as i32)],
        )
        .unwrap_or_else(|_| fallback_daily_avg(total_cents, days))
    }

    /// Forecast total based on current spending rate
    pub fn calculate_forecast(&mut self, current: u64, elapsed: u32, total: u32) -> u64 {
        self.call_u64(
            "calculate_forecast",
            &[
                wasmtime::Val::I64(current as i64),
                wasmtime::Val::I32(elapsed as i32),
                wasmtime::Val::I32(total as i32),
            ],
        )
        .unwrap_or_else(|_| fallback_forecast(current, elapsed, total))
    }

    /// Format token count to human-readable string (e.g. "1.2M")
    pub fn format_tokens(&mut self, count: u64) -> String {
        self.call_format_fn("format_tokens", wasmtime::Val::I64(count as i64))
            .unwrap_or_else(|_| fallback_format_tokens(count))
    }

    /// Format cost in cents to USD string (e.g. "$12.34")
    pub fn format_cost(&mut self, cents: u64) -> String {
        self.call_format_fn("format_cost_usd", wasmtime::Val::I64(cents as i64))
            .unwrap_or_else(|_| fallback_format_cost(cents))
    }

    /// Format seconds to human-readable duration (e.g. "2d 5h")
    pub fn format_duration(&mut self, secs: i64) -> String {
        self.call_format_fn("format_duration", wasmtime::Val::I64(secs))
            .unwrap_or_else(|_| fallback_format_duration(secs))
    }

    // ── Internal call helpers ──

    fn call_f64(&mut self, name: &str, args: &[wasmtime::Val]) -> Result<f64> {
        let func = self
            .instance
            .get_func(&mut self.store, name)
            .with_context(|| format!("WASM export '{name}' not found"))?;
        let mut results = [wasmtime::Val::F64(0u64)];
        func.call(&mut self.store, args, &mut results)?;
        match results[0] {
            wasmtime::Val::F64(bits) => Ok(f64::from_bits(bits)),
            _ => anyhow::bail!("Unexpected return type from {name}"),
        }
    }

    fn call_i64(&mut self, name: &str, args: &[wasmtime::Val]) -> Result<i64> {
        let func = self
            .instance
            .get_func(&mut self.store, name)
            .with_context(|| format!("WASM export '{name}' not found"))?;
        let mut results = [wasmtime::Val::I64(0)];
        func.call(&mut self.store, args, &mut results)?;
        match results[0] {
            wasmtime::Val::I64(v) => Ok(v),
            _ => anyhow::bail!("Unexpected return type from {name}"),
        }
    }

    fn call_u64(&mut self, name: &str, args: &[wasmtime::Val]) -> Result<u64> {
        let v = self.call_i64(name, args)?;
        Ok(v as u64)
    }

    /// Call a Zig format function: fn(value, buf_ptr, buf_len) -> bytes_written
    /// The Zig functions write formatted text into a buffer in WASM memory.
    fn call_format_fn(&mut self, name: &str, value: wasmtime::Val) -> Result<String> {
        let func = self
            .instance
            .get_func(&mut self.store, name)
            .with_context(|| format!("WASM export '{name}' not found"))?;

        let args = [
            value,
            wasmtime::Val::I32(WASM_BUF_OFFSET as i32),
            wasmtime::Val::I32(WASM_BUF_SIZE as i32),
        ];
        let mut results = [wasmtime::Val::I32(0)];
        func.call(&mut self.store, &args, &mut results)?;

        let bytes_written = match results[0] {
            wasmtime::Val::I32(v) => v as u32 as usize,
            _ => anyhow::bail!("Unexpected return type from {name}"),
        };

        if bytes_written == 0 {
            return Ok(String::new());
        }

        let data = self.memory.data(&self.store);
        let start = WASM_BUF_OFFSET as usize;
        let end = start + bytes_written;
        if end > data.len() {
            anyhow::bail!("WASM memory out of bounds: offset={start}, len={bytes_written}");
        }

        let s = std::str::from_utf8(&data[start..end])
            .context("WASM returned invalid UTF-8")?;
        Ok(s.to_string())
    }
}

// ── Pure Rust fallback implementations ──

pub fn fallback_usage_pct(used: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (used as f64 / total as f64) * 100.0
}

pub fn fallback_time_remaining(reset_ts: i64, now_ts: i64) -> i64 {
    let diff = reset_ts - now_ts;
    if diff < 0 { 0 } else { diff }
}

pub fn fallback_daily_avg(total_cents: u64, days: u32) -> u64 {
    if days == 0 {
        return 0;
    }
    total_cents / days as u64
}

pub fn fallback_forecast(current: u64, elapsed: u32, total: u32) -> u64 {
    if elapsed == 0 {
        return 0;
    }
    (current as f64 / elapsed as f64 * total as f64) as u64
}

pub fn fallback_format_tokens(count: u64) -> String {
    if count >= 1_000_000_000 {
        format!("{:.1}B", count as f64 / 1_000_000_000.0)
    } else if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1_000 {
        format!("{:.1}K", count as f64 / 1_000.0)
    } else {
        count.to_string()
    }
}

pub fn fallback_format_cost(cents: u64) -> String {
    let dollars = cents as f64 / 100.0;
    format!("${:.2}", dollars)
}

pub fn fallback_format_duration(secs: i64) -> String {
    if secs <= 0 {
        return "0s".into();
    }
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}
