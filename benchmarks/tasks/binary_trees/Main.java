public class Main {
  static final class Node {
    Node left, right;
    Node(Node l, Node r) { left = l; right = r; }
  }
  static Node make(int d) {
    if (d == 0) return new Node(null, null);
    return new Node(make(d - 1), make(d - 1));
  }
  static long check(Node n) {
    if (n.left == null) return 1;
    return 1 + check(n.left) + check(n.right);
  }
  public static void main(String[] args) {
    final int MIN_D = 4, MAX_D = 13;
    long stretch = check(make(MAX_D + 1));
    Node longLived = make(MAX_D);
    long total = 0;
    for (int depth = MIN_D; depth <= MAX_D; depth += 2) {
      int iterations = 1 << (MAX_D - depth + MIN_D);
      long acc = 0;
      for (int i = 0; i < iterations; i++) acc += check(make(depth));
      total += acc;
    }
    total += check(longLived);
    System.out.println(stretch + " " + total);
  }
}
