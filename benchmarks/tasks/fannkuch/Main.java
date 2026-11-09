public class Main {
  static final int N = 9;
  static long[] perm = new long[N];
  static long best, checksum, sign;
  static long flips(long[] p) {
    long[] q = p.clone();
    long count = 0;
    while (q[0] != 0) {
      int k = (int) q[0];
      int i = 0, j = k;
      while (i < j) { long t = q[i]; q[i] = q[j]; q[j] = t; i++; j--; }
      count++;
    }
    return count;
  }
  static void rec(int k) {
    if (k == 1) {
      if (perm[0] != 0) {
        long got = flips(perm);
        checksum += sign * got;
        if (got > best) best = got;
      }
      sign = -sign;
      return;
    }
    rec(k - 1);
    for (int i = 0; i < k - 1; i++) {
      long t = perm[0];
      for (int j = 0; j < k - 1; j++) perm[j] = perm[j + 1];
      perm[k - 1] = t;
      rec(k - 1);
    }
  }
  public static void main(String[] args) {
    for (int i = 0; i < N; i++) perm[i] = i;
    sign = 1;
    rec(N);
    System.out.println(checksum + " " + best);
  }
}
