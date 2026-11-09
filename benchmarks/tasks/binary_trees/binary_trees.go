package main

import "fmt"

const minD = 4
const maxD = 13

type node struct {
	left  *node
	right *node
}

func makeNode(depth int) *node {
	if depth == 0 {
		return &node{}
	}
	return &node{left: makeNode(depth - 1), right: makeNode(depth - 1)}
}

func check(n *node) int64 {
	if n.left == nil {
		return 1
	}
	return 1 + check(n.left) + check(n.right)
}

func main() {
	stretch := check(makeNode(maxD + 1))
	longLived := makeNode(maxD)
	total := int64(0)
	for depth := minD; depth <= maxD; depth += 2 {
		iterations := 1 << (maxD - depth + minD)
		acc := int64(0)
		for i := 0; i < iterations; i++ {
			acc += check(makeNode(depth))
		}
		total += acc
	}
	total += check(longLived)
	fmt.Println(stretch, total)
}
