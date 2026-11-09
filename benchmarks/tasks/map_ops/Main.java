import java.util.HashMap;
public class Main {
  static long lcg(long x) { return (x * 1664525L + 1013904223L) & 0xffffffffL; }
  public static void main(String[] args) {
    final int N = 800000;
    HashMap<Long, Long> table = new HashMap<Long, Long>();
    long x = 1;
    for (long i = 0; i < N; i++) { x = lcg(x); table.put(x, i); }
    long found = 0, total = 0;
    x = 1;
    for (long i = 0; i < N; i++) {
      x = lcg(x);
      Long v = table.get(x);
      if (v != null) { found++; total += v; }
    }
    System.out.println(found + " " + total + " " + table.size());
  }
}
