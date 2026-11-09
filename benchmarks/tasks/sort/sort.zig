const std = @import("std");
fn emit(io: std.Io, comptime fmt: []const u8, args: anytype) void {
    var buf: [256]u8 = undefined;
    var w = std.Io.File.stdout().writer(io, &buf);
    w.interface.print(fmt, args) catch {};
    w.interface.flush() catch {};
}

fn lcg(x: i64) i64 {
    return (x * 1664525 + 1013904223) & 0xffffffff;
}
fn less(_: void, a: i64, b: i64) bool {
    return a < b;
}
pub fn main(init: std.process.Init) !void {
    const N: usize = 1000000;
    const vals = std.heap.page_allocator.alloc(i64, N) catch unreachable;
    defer std.heap.page_allocator.free(vals);
    var x: i64 = 1;
    for (vals) |*slot| {
        x = lcg(x);
        slot.* = x;
    }
    std.mem.sort(i64, vals, {}, less);
    var acc: i64 = 0;
    const step = N / 10;
    var i: usize = 0;
    while (i < N) : (i += step) {
        acc = (acc + vals[i] * @as(i64, @intCast(i + 1))) & 0x7fffffff;
    }
    emit(init.io, "{d} {d} {d} {d}\n", .{ vals[0], vals[N / 2], vals[N - 1], acc });
}
