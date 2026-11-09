proc fib(n: int): int =
  if n < 2: return n
  fib(n - 1) + fib(n - 2)

echo fib(36)
