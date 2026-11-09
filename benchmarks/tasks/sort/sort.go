package main

import (
	"fmt"
	"sort"
)

const n = 1000000

func lcg(x int64) int64 { return (x*1664525 + 1013904223) & 0xffffffff }

func main() {
	x := int64(1)
	vals := make([]int64, n)
	for i := 0; i < n; i++ {
		x = lcg(x)
		vals[i] = x
	}
	sort.Slice(vals, func(i, j int) bool { return vals[i] < vals[j] })
	acc := int64(0)
	step := n / 10
	for i := 0; i < n; i += step {
		acc = (acc + vals[i]*int64(i+1)) & 0x7fffffff
	}
	fmt.Println(vals[0], vals[n/2], vals[n-1], acc)
}
