const MIN_D = 4
const MAX_D = 13

type Node = ref object
  left: Node
  right: Node

proc make(depth: int): Node =
  if depth == 0: Node(left: nil, right: nil)
  else: Node(left: make(depth - 1), right: make(depth - 1))

proc check(n: Node): int =
  if n.left == nil: 1
  else: 1 + check(n.left) + check(n.right)

let stretch = check(make(MAX_D + 1))
let longLived = make(MAX_D)
var total = 0
var depth = MIN_D
while depth <= MAX_D:
  let iterations = 1 shl (MAX_D - depth + MIN_D)
  var acc = 0
  for i in 0..<iterations:
    acc += check(make(depth))
  total += acc
  depth += 2
total += check(longLived)
echo stretch, " ", total
