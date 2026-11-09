const std = @import("std");
fn emit(io: std.Io, comptime fmt: []const u8, args: anytype) void {
    var buf: [256]u8 = undefined;
    var w = std.Io.File.stdout().writer(io, &buf);
    w.interface.print(fmt, args) catch {};
    w.interface.flush() catch {};
}

pub fn main(init: std.process.Init) !void {
    const N: usize = 5000000;
    var mark = std.heap.page_allocator.alloc(u8, N + 1) catch unreachable;
    defer std.heap.page_allocator.free(mark);
    @memset(mark, 1);
    mark[0] = 0;
    mark[1] = 0;
    var count: i64 = 0;
    var last: i64 = 0;
    var i: usize = 2;
    while (i <= N) : (i += 1) {
        if (mark[i] != 0) {
            count += 1;
            last = @intCast(i);
            var j = i * i;
            while (j <= N) : (j += i) mark[j] = 0;
        }
    }
    emit(init.io, "{d} {d}\n", .{ count, last });
}
