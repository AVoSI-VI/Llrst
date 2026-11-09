public class Main {
  public static void main(String[] args) {
    final int N = 5000000;
    byte[] mark = new byte[N + 1];
    for (int i = 0; i <= N; i++) mark[i] = 1;
    mark[0] = 0;
    mark[1] = 0;
    long count = 0;
    long last = 0;
    for (int i = 2; i <= N; i++) {
      if (mark[i] != 0) {
        count++;
        last = i;
        for (long j = (long) i * i; j <= N; j += i) mark[(int) j] = 0;
      }
    }
    System.out.println(count + " " + last);
  }
}
