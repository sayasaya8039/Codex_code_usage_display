/// Calculate usage percentage (0.0 to 100.0).
/// Returns 0.0 if total is 0.
pub export fn calculate_usage_percentage(used: u64, total: u64) callconv(.C) f64 {
    if (total == 0) return 0.0;
    const used_f: f64 = @floatFromInt(used);
    const total_f: f64 = @floatFromInt(total);
    const result = (used_f / total_f) * 100.0;
    if (result > 100.0) return 100.0;
    return result;
}

/// Calculate time remaining in seconds.
/// Returns negative value if already expired.
pub export fn calculate_time_remaining(reset_timestamp: i64, current_timestamp: i64) callconv(.C) i64 {
    return reset_timestamp - current_timestamp;
}

/// Calculate daily average cost in cents.
/// Returns 0 if days is 0.
pub export fn calculate_daily_average(total_cost_cents: u64, days: u32) callconv(.C) u64 {
    if (days == 0) return 0;
    return total_cost_cents / @as(u64, days);
}

/// Forecast total usage by end of period.
/// Extrapolates current_usage over total_days based on days_elapsed.
/// Returns current_usage if days_elapsed is 0.
pub export fn calculate_forecast(current_usage: u64, days_elapsed: u32, total_days: u32) callconv(.C) u64 {
    if (days_elapsed == 0 or total_days == 0) return current_usage;
    const usage_f: f64 = @floatFromInt(current_usage);
    const elapsed_f: f64 = @floatFromInt(days_elapsed);
    const total_f: f64 = @floatFromInt(total_days);
    const forecast = (usage_f / elapsed_f) * total_f;
    // Clamp to u64 max
    if (forecast >= @as(f64, @floatFromInt(@as(u64, std.math.maxInt(u64))))) {
        return std.math.maxInt(u64);
    }
    return @intFromFloat(forecast);
}

/// Forecast total cost in cents by end of period.
/// Same logic as calculate_forecast but for cost values.
pub export fn calculate_cost_forecast(current_cost_cents: u64, days_elapsed: u32, total_days: u32) callconv(.C) u64 {
    return calculate_forecast(current_cost_cents, days_elapsed, total_days);
}

const std = @import("std");
