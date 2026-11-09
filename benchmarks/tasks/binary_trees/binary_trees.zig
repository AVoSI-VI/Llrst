const std = @import("std");
fn emit(io: std.Io, comptime fmt: []const u8, args: anytype) void {
    var buf: [256]u8 = undefined;
    var w = std.Io.File.stdout().writer(io, &buf);
    w.interface.print(fmt, args) catch {};
    w.interface.flush() catch {};
}

const Node = struct { left: ?*Node, right: ?*Node };
fn make(alloc: std.mem.Allocator, depth: i32) *Node {
    const n = alloc.create(Node) catch unreachable;
    if (depth == 0) {
        n.* = .{ .left = null, .right = null };
    } else {
        n.* = .{ .left = make(alloc, depth - 1), .right = make(alloc, depth - 1) };
    }
    return n;
}
fn check(n: *const Node) i64 {
    if (n.left == null) return 1;
    return 1 + check(n.left.?) + check(n.right.?);
}
fn release(alloc: std.mem.Allocator, n: *Node) void {
    if (n.left) |l| release(alloc, l);
    if (n.right) |r| release(alloc, r);
    alloc.destroy(n);
}
pub fn main(init: std.process.Init) !void {
    const MIN_D = 4;
    const MAX_D = 13;
    const alloc = std.heap.page_allocator;
    const stretch_n = make(alloc, MAX_D + 1);
    const stretch = check(stretch_n);
    release(alloc, stretch_n);
    const long_lived = make(alloc, MAX_D);
    var total: i64 = 0;
    var depth: i32 = MIN_D;
    while (depth <= MAX_D) : (depth += 2) {
        const iterations: i32 = @as(i32, 1) << @intCast(MAX_D - depth + MIN_D);
        var acc: i64 = 0;
        var i: i32 = 0;
        while (i < iterations) : (i += 1) {
            const t = make(alloc, depth);
            acc += check(t);
            release(alloc, t);
        }
        total += acc;
    }
    total += check(long_lived);
    release(alloc, long_lived);
    emit(init.io, "{d} {d}\n", .{ stretch, total });
}
