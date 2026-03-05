const std = @import("std");

/// Format token count into human-readable string.
/// Examples: 1234567 → "1.23M", 12345 → "12.3K", 999 → "999"
/// Returns the number of bytes written to buf.
pub export fn format_tokens(count: u64, buf: [*]u8, buf_len: u32) callconv(.C) u32 {
    var fbs = std.io.fixedBufferStream(buf[0..buf_len]);
    const writer = fbs.writer();

    if (count >= 1_000_000_000) {
        const val: f64 = @as(f64, @floatFromInt(count)) / 1_000_000_000.0;
        writer.print("{d:.2}B", .{val}) catch return 0;
    } else if (count >= 1_000_000) {
        const val: f64 = @as(f64, @floatFromInt(count)) / 1_000_000.0;
        writer.print("{d:.2}M", .{val}) catch return 0;
    } else if (count >= 1_000) {
        const val: f64 = @as(f64, @floatFromInt(count)) / 1_000.0;
        writer.print("{d:.1}K", .{val}) catch return 0;
    } else {
        writer.print("{d}", .{count}) catch return 0;
    }

    return @intCast(fbs.pos);
}

/// Format cents to USD string.
/// Example: 1234 → "$12.34", 5 → "$0.05"
/// Returns the number of bytes written to buf.
pub export fn format_cost_usd(cents: u64, buf: [*]u8, buf_len: u32) callconv(.C) u32 {
    var fbs = std.io.fixedBufferStream(buf[0..buf_len]);
    const writer = fbs.writer();

    const dollars = cents / 100;
    const remainder = cents % 100;

    writer.print("${d}.{d:0>2}", .{ dollars, remainder }) catch return 0;

    return @intCast(fbs.pos);
}

/// Format seconds into human-readable duration.
/// Example: 3661 → "1h 1m 1s", 59 → "59s", -120 → "-2m 0s"
/// Returns the number of bytes written to buf.
pub export fn format_duration(seconds: i64, buf: [*]u8, buf_len: u32) callconv(.C) u32 {
    var fbs = std.io.fixedBufferStream(buf[0..buf_len]);
    const writer = fbs.writer();

    var secs = seconds;
    if (secs < 0) {
        writer.writeByte('-') catch return 0;
        secs = -secs;
    }

    const abs: u64 = @intCast(secs);
    const hours = abs / 3600;
    const minutes = (abs % 3600) / 60;
    const remaining_secs = abs % 60;

    if (hours > 0) {
        writer.print("{d}h {d}m {d}s", .{ hours, minutes, remaining_secs }) catch return 0;
    } else if (minutes > 0) {
        writer.print("{d}m {d}s", .{ minutes, remaining_secs }) catch return 0;
    } else {
        writer.print("{d}s", .{remaining_secs}) catch return 0;
    }

    return @intCast(fbs.pos);
}

/// Format a floating point percentage value.
/// Example: 78.5 → "78.5%", 100.0 → "100.0%"
/// Returns the number of bytes written to buf.
pub export fn format_percentage(value: f64, buf: [*]u8, buf_len: u32) callconv(.C) u32 {
    var fbs = std.io.fixedBufferStream(buf[0..buf_len]);
    const writer = fbs.writer();

    writer.print("{d:.1}%", .{value}) catch return 0;

    return @intCast(fbs.pos);
}
