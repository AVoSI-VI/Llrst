const std = @import("std");
fn emit(io: std.Io, comptime fmt: []const u8, args: anytype) void {
    var buf: [256]u8 = undefined;
    var w = std.Io.File.stdout().writer(io, &buf);
    w.interface.print(fmt, args) catch {};
    w.interface.flush() catch {};
}

pub fn main(init: std.process.Init) !void {
    const W: i64 = 600;
    const LIM: i64 = 50;
    var count: i64 = 0;
    var y: i64 = 0;
    while (y < W) : (y += 1) {
        var x: i64 = 0;
        while (x < W) : (x += 1) {
            const cr = 2.0 * @as(f64, @floatFromInt(x)) / @as(f64, @floatFromInt(W)) - 1.5;
            const ci = 2.0 * @as(f64, @floatFromInt(y)) / @as(f64, @floatFromInt(W)) - 1.0;
            var zr: f64 = 0;
            var zi: f64 = 0;
            var inside = true;
            var k: i64 = 0;
            while (k < LIM) : (k += 1) {
                const zr2 = zr * zr - zi * zi + cr;
                zi = 2.0 * zr * zi + ci;
                zr = zr2;
                if (zr * zr + zi * zi > 4.0) {
                    inside = false;
                    break;
                }
            }
            if (inside) count += 1;
        }
    }
    emit(init.io, "{d}\n", .{count});
}
