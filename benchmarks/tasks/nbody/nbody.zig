const std = @import("std");
fn emit(io: std.Io, comptime fmt: []const u8, args: anytype) void {
    var buf: [256]u8 = undefined;
    var w = std.Io.File.stdout().writer(io, &buf);
    w.interface.print(fmt, args) catch {};
    w.interface.flush() catch {};
}

const STEPS: i64 = 50000;
const DT: f64 = 0.01;
const PI: f64 = 3.141592653589793;
const DAYS: f64 = 365.24;
fn nsqrt(x: f64) f64 {
    if (x == 0) return 0;
    var guess: f64 = if (x > 1) x * 0.5 else 1;
    var i: i32 = 0;
    while (i < 8) : (i += 1) guess = 0.5 * (guess + x / guess);
    return guess;
}
fn advance(b: *[5][7]f64) void {
    var i: usize = 0;
    while (i < 5) : (i += 1) {
        var j: usize = i + 1;
        while (j < 5) : (j += 1) {
            const dx = b[i][0] - b[j][0];
            const dy = b[i][1] - b[j][1];
            const dz = b[i][2] - b[j][2];
            const dist = nsqrt(dx * dx + dy * dy + dz * dz);
            const mag = DT / (dist * dist * dist);
            const im = b[i][6];
            const jm = b[j][6];
            b[i][3] -= dx * jm * mag;
            b[i][4] -= dy * jm * mag;
            b[i][5] -= dz * jm * mag;
            b[j][3] += dx * im * mag;
            b[j][4] += dy * im * mag;
            b[j][5] += dz * im * mag;
        }
    }
    for (b) |*body| {
        body[0] += DT * body[3];
        body[1] += DT * body[4];
        body[2] += DT * body[5];
    }
}
fn energy(b: *const [5][7]f64) f64 {
    var e: f64 = 0;
    var i: usize = 0;
    while (i < 5) : (i += 1) {
        e += 0.5 * b[i][6] * (b[i][3] * b[i][3] + b[i][4] * b[i][4] + b[i][5] * b[i][5]);
        var j: usize = i + 1;
        while (j < 5) : (j += 1) {
            const dx = b[i][0] - b[j][0];
            const dy = b[i][1] - b[j][1];
            const dz = b[i][2] - b[j][2];
            e -= b[i][6] * b[j][6] / nsqrt(dx * dx + dy * dy + dz * dz);
        }
    }
    return e;
}
pub fn main(init: std.process.Init) !void {
    const sm = 4.0 * PI * PI;
    var b = [5][7]f64{
        .{ 0, 0, 0, 0, 0, 0, sm },
        .{ 4.84143144246472090, -1.16032004402742839, -0.103622044471123109, 0.00166007664274403694 * DAYS, 0.00769901118419740425 * DAYS, -0.0000690460016972063023 * DAYS, 0.000954791938424326609 * sm },
        .{ 8.34336671824457987, 4.12479856412430479, -0.403523417114321381, -0.00276742510726862411 * DAYS, 0.00499852801234917238 * DAYS, 0.0000230417297573763929 * DAYS, 0.000285885980666130812 * sm },
        .{ 12.8943695621391310, -15.1111514016986312, -0.223307578892655734, 0.00296460137564761618 * DAYS, 0.00237847173959480950 * DAYS, -0.0000296589568540237556 * DAYS, 0.0000436624404335156298 * sm },
        .{ 15.3796971148509165, -25.9193146099879641, 0.179258772950371181, 0.00268067772490389322 * DAYS, 0.00162824170038242295 * DAYS, -0.0000951592254519715870 * DAYS, 0.0000515138902046611451 * sm },
    };
    var px: f64 = 0;
    var py: f64 = 0;
    var pz: f64 = 0;
    for (b) |body| {
        px += body[3] * body[6];
        py += body[4] * body[6];
        pz += body[5] * body[6];
    }
    b[0][3] = -px / sm;
    b[0][4] = -py / sm;
    b[0][5] = -pz / sm;
    var s: i64 = 0;
    while (s < STEPS) : (s += 1) advance(&b);
    emit(init.io, "{d}\n", .{@as(i64, @intFromFloat(energy(&b) * 10000.0))});
}
