import java.util.Arrays;
public class Main {
  static long lcg(long x) { return (x * 1664525L + 1013904223L) & 0xffffffffL; }
  public static void main(String[] args) {
    final int N = 1000000;
    long[] vals = new long[N];
    long x = 1;
    for (int i = 0; i < N; i++) { x = lcg(x); vals[i] = x; }
    Arrays.sort(vals);
    long acc = 0;
    int step = N / 10;
    for (int i = 0; i < N; i += step) acc = (acc + vals[i] * (i + 1)) & 0x7fffffffL;
    System.out.println(vals[0] + " " + vals[N / 2] + " " + vals[N - 1] + " " + acc);
  }
}
