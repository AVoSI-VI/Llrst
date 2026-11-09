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
    text.append(alloc, '[') catch unreachable;
    var i: usize = 0;
    while (i < N) : (i += 1) {
        if (i > 0) text.append(alloc, ',') catch unreachable;
        var buf: [64]u8 = undefined;
        const s = std.fmt.bufPrint(&buf, "{{\"id\":{d},\"ok\":true}}", .{i}) catch unreachable;
        text.appendSlice(alloc, s) catch unreachable;
    }
    text.append(alloc, ']') catch unreachable;
    var sum: i64 = 0;
    var nums: i64 = 0;
    var k: usize = 0;
    const bytes = text.items;
    while (k < bytes.len) {
        const c = bytes[k];
        if (c >= '0' and c <= '9') {
            var v: i64 = 0;
            while (k < bytes.len and bytes[k] >= '0' and bytes[k] <= '9') : (k += 1) {
                v = v * 10 + @as(i64, bytes[k] - '0');
            }
            sum += v;
            nums += 1;
        } else k += 1;
    }
    emit(init.io, "{d} {d}\n", .{ sum, nums });
}
