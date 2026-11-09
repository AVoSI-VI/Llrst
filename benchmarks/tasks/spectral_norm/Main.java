public class Main {
  static double aval(int i, int j) {
    int ij = i + j;
    return 1.0 / ((ij * (ij + 1)) / 2 + i + 1);
  }
  static double[] mul(double[] v, boolean trans) {
    int n = v.length;
    double[] out = new double[n];
    for (int i = 0; i < n; i++) {
      double s = 0;
      for (int j = 0; j < n; j++) s += (trans ? aval(j, i) : aval(i, j)) * v[j];
      out[i] = s;
    }
    return out;
  }
  public static void main(String[] args) {
    final int N = 500, ITERS = 10;
    double[] u = new double[N];
    for (int i = 0; i < N; i++) u[i] = 1;
    for (int k = 0; k < ITERS; k++) {
      double[] v = mul(u, false);
      u = mul(v, true);
    }
    long acc = 0;
    for (double x : u) acc += (long) (x * 1000000.0);
    System.out.println(acc);
  }
}
