const std = @import("std");
fn emit(io: std.Io, comptime fmt: []const u8, args: anytype) void {
    var buf: [256]u8 = undefined;
    var w = std.Io.File.stdout().writer(io, &buf);
    w.interface.print(fmt, args) catch {};
    w.interface.flush() catch {};
}

fn aval(i: i64, j: i64) f64 {
    const ij = i + j;
    return 1.0 / @as(f64, @floatFromInt(@divTrunc(ij * (ij + 1), 2) + i + 1));
}
fn mul(v: []const f64, outv: []f64, trans: bool) void {
    var i: usize = 0;
    while (i < v.len) : (i += 1) {
        var s: f64 = 0;
        var j: usize = 0;
        while (j < v.len) : (j += 1) {
            const a = if (trans) aval(@intCast(j), @intCast(i)) else aval(@intCast(i), @intCast(j));
            s += a * v[j];
        }
        outv[i] = s;
    }
}
pub fn main(init: std.process.Init) !void {
    const N: usize = 500;
    const ITERS = 10;
    const u = std.heap.page_allocator.alloc(f64, N) catch unreachable;
    const v = std.heap.page_allocator.alloc(f64, N) catch unreachable;
    const w = std.heap.page_allocator.alloc(f64, N) catch unreachable;
    defer std.heap.page_allocator.free(u);
    defer std.heap.page_allocator.free(v);
    defer std.heap.page_allocator.free(w);
    @memset(u, 1);
    var k: usize = 0;
    while (k < ITERS) : (k += 1) {
        mul(u, v, false);
        mul(v, w, true);
        @memcpy(u, w);
    }
    var acc: i64 = 0;
    for (u) |x| acc += @intFromFloat(x * 1000000.0);
    emit(init.io, "{d}\n", .{acc});
}
