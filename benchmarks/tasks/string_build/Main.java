public class Main {
  public static void main(String[] args) {
    final int N = 800000;
    StringBuilder b = new StringBuilder();
    for (int i = 0; i < N; i++) {
      if (i > 0) b.append(',');
      b.append(i);
    }
    String text = b.toString();
    long acc = 0;
    for (int i = 0; i < text.length(); i++) acc = (acc * 131 + text.charAt(i)) & 0xffffffffL;
    System.out.println(text.length() + " " + acc);
  }
}
