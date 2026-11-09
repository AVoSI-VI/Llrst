public class Main {
  static final int STEPS = 50000;
  static final double DT = 0.01;
  static final double PI = 3.141592653589793;
  static final double DAYS = 365.24;
  static double nsqrt(double x) {
    if (x == 0) return 0;
    double g = x > 1 ? x * 0.5 : 1;
    for (int i = 0; i < 8; i++) g = 0.5 * (g + x / g);
    return g;
  }
  static void advance(double[][] b) {
    for (int i = 0; i < b.length; i++) {
      for (int j = i + 1; j < b.length; j++) {
        double dx = b[i][0] - b[j][0], dy = b[i][1] - b[j][1], dz = b[i][2] - b[j][2];
        double dist = nsqrt(dx * dx + dy * dy + dz * dz);
        double mag = DT / (dist * dist * dist);
        double im = b[i][6], jm = b[j][6];
        b[i][3] -= dx * jm * mag; b[i][4] -= dy * jm * mag; b[i][5] -= dz * jm * mag;
        b[j][3] += dx * im * mag; b[j][4] += dy * im * mag; b[j][5] += dz * im * mag;
      }
    }
    for (double[] body : b) {
      body[0] += DT * body[3]; body[1] += DT * body[4]; body[2] += DT * body[5];
    }
  }
  static double energy(double[][] b) {
    double e = 0;
    for (int i = 0; i < b.length; i++) {
      e += 0.5 * b[i][6] * (b[i][3] * b[i][3] + b[i][4] * b[i][4] + b[i][5] * b[i][5]);
      for (int j = i + 1; j < b.length; j++) {
        double dx = b[i][0] - b[j][0], dy = b[i][1] - b[j][1], dz = b[i][2] - b[j][2];
        e -= b[i][6] * b[j][6] / nsqrt(dx * dx + dy * dy + dz * dz);
      }
    }
    return e;
  }
  public static void main(String[] args) {
    double sm = 4 * PI * PI;
    double[][] b = {
      {0, 0, 0, 0, 0, 0, sm},
      {4.84143144246472090, -1.16032004402742839, -0.103622044471123109, 0.00166007664274403694 * DAYS, 0.00769901118419740425 * DAYS, -0.0000690460016972063023 * DAYS, 0.000954791938424326609 * sm},
      {8.34336671824457987, 4.12479856412430479, -0.403523417114321381, -0.00276742510726862411 * DAYS, 0.00499852801234917238 * DAYS, 0.0000230417297573763929 * DAYS, 0.000285885980666130812 * sm},
      {12.8943695621391310, -15.1111514016986312, -0.223307578892655734, 0.00296460137564761618 * DAYS, 0.00237847173959480950 * DAYS, -0.0000296589568540237556 * DAYS, 0.0000436624404335156298 * sm},
      {15.3796971148509165, -25.9193146099879641, 0.179258772950371181, 0.00268067772490389322 * DAYS, 0.00162824170038242295 * DAYS, -0.0000951592254519715870 * DAYS, 0.0000515138902046611451 * sm},
    };
    double px = 0, py = 0, pz = 0;
    for (double[] body : b) { px += body[3] * body[6]; py += body[4] * body[6]; pz += body[5] * body[6]; }
    b[0][3] = -px / sm; b[0][4] = -py / sm; b[0][5] = -pz / sm;
    for (int s = 0; s < STEPS; s++) advance(b);
    System.out.println((long) (energy(b) * 10000.0));
  }
}
