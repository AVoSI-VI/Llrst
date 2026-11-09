const std = @import("std");
fn emit(io: std.Io, comptime fmt: []const u8, args: anytype) void {
    var buf: [256]u8 = undefined;
    var w = std.Io.File.stdout().writer(io, &buf);
    w.interface.print(fmt, args) catch {};
    w.interface.flush() catch {};
}

fn fib(n: i64) i64 {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}
pub fn main(init: std.process.Init) !void {
    emit(init.io, "{d}\n", .{fib(36)});
}
