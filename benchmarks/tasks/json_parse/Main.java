public class Main {
  public static void main(String[] args) {
    final int N = 800000;
    StringBuilder b = new StringBuilder();
    b.append('[');
    for (int i = 0; i < N; i++) {
      if (i > 0) b.append(',');
      b.append("{\"id\":").append(i).append(",\"ok\":true}");
    }
    b.append(']');
    String text = b.toString();
    long sum = 0, nums = 0;
    for (int i = 0; i < text.length(); i++) {
      char c = text.charAt(i);
      if (c >= '0' && c <= '9') {
        long v = 0;
        while (i < text.length() && text.charAt(i) >= '0' && text.charAt(i) <= '9') {
          v = v * 10 + (text.charAt(i) - '0');
          i++;
        }
        i--;
        sum += v;
        nums++;
      }
    }
    System.out.println(sum + " " + nums);
  }
}
