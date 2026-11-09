const std = @import("std");
fn emit(io: std.Io, comptime fmt: []const u8, args: anytype) void {
    var buf: [256]u8 = undefined;
    var w = std.Io.File.stdout().writer(io, &buf);
    w.interface.print(fmt, args) catch {};
    w.interface.flush() catch {};
}

const N: usize = 9;
var perm: [N]i64 = undefined;
var best: i64 = 0;
var checksum: i64 = 0;
var sign: i64 = 1;
fn flips() i64 {
    var q: [N]i64 = perm;
    var count: i64 = 0;
    while (q[0] != 0) {
        const k: usize = @intCast(q[0]);
        var i: usize = 0;
        var j: usize = k;
        while (i < j) {
            const t = q[i];
            q[i] = q[j];
            q[j] = t;
            i += 1;
            j -= 1;
        }
        count += 1;
    }
    return count;
}
fn rec(k: usize) void {
    if (k == 1) {
        if (perm[0] != 0) {
            const got = flips();
            checksum += sign * got;
            if (got > best) best = got;
        }
        sign = -sign;
        return;
    }
    rec(k - 1);
    var i: usize = 0;
    while (i < k - 1) : (i += 1) {
        const t = perm[0];
        var j: usize = 0;
        while (j < k - 1) : (j += 1) perm[j] = perm[j + 1];
        perm[k - 1] = t;
        rec(k - 1);
    }
}
pub fn main(init: std.process.Init) !void {
    for (0..N) |i| perm[i] = @intCast(i);
    rec(N);
    emit(init.io, "{d} {d}\n", .{ checksum, best });
}
