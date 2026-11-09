const std = @import("std");
fn emit(io: std.Io, comptime fmt: []const u8, args: anytype) void {
    var buf: [256]u8 = undefined;
    var w = std.Io.File.stdout().writer(io, &buf);
    w.interface.print(fmt, args) catch {};
    w.interface.flush() catch {};
}

pub fn main(init: std.process.Init) !void {
    const N: usize = 800000;
    const alloc = std.heap.page_allocator;
    var text: std.ArrayList(u8) = .empty;
    defer text.deinit(alloc);
    var i: usize = 0;
    while (i < N) : (i += 1) {
        if (i > 0) text.append(alloc, ',') catch unreachable;
        var buf: [32]u8 = undefined;
        const s = std.fmt.bufPrint(&buf, "{d}", .{i}) catch unreachable;
        text.appendSlice(alloc, s) catch unreachable;
    }
    var acc: i64 = 0;
    for (text.items) |byte| {
        acc = (acc * 131 + byte) & 0xffffffff;
    }
    emit(init.io, "{d} {d}\n", .{ text.items.len, acc });
}
