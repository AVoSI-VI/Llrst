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
pub fn main(init: std.process.Init) !void {
    const N: i64 = 800000;
    var table = std.AutoHashMap(i64, i64).init(std.heap.page_allocator);
    defer table.deinit();
    var x: i64 = 1;
    var i: i64 = 0;
    while (i < N) : (i += 1) {
        x = lcg(x);
        table.put(x, i) catch unreachable;
    }
    var found: i64 = 0;
    var total: i64 = 0;
    x = 1;
    i = 0;
    while (i < N) : (i += 1) {
        x = lcg(x);
        if (table.get(x)) |v| {
            found += 1;
            total += v;
        }
    }
    emit(init.io, "{d} {d} {d}\n", .{ found, total, table.count() });
}
