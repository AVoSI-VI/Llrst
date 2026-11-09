public class Main {
  public static void main(String[] args) {
    final int W = 600;
    final int LIM = 50;
    long count = 0;
    for (int y = 0; y < W; y++) {
      for (int x = 0; x < W; x++) {
        double cr = 2.0 * x / W - 1.5;
        double ci = 2.0 * y / W - 1.0;
        double zr = 0, zi = 0;
        boolean inside = true;
        for (int k = 0; k < LIM; k++) {
          double zr2 = zr * zr - zi * zi + cr;
          zi = 2.0 * zr * zi + ci;
          zr = zr2;
          if (zr * zr + zi * zi > 4.0) { inside = false; break; }
        }
        if (inside) count++;
      }
    }
    System.out.println(count);
  }
}
