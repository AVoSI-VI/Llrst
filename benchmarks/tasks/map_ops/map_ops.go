package main

import "fmt"

const n = 800000

func lcg(x int64) int64 { return (x*1664525 + 1013904223) & 0xffffffff }

func main() {
	table := make(map[int64]int64, n)
	x := int64(1)
	for i := int64(0); i < n; i++ {
		x = lcg(x)
		table[x] = i
	}
	found := int64(0)
	total := int64(0)
	x = 1
	for i := int64(0); i < n; i++ {
		x = lcg(x)
		if v, ok := table[x]; ok {
			found++
			total += v
		}
	}
	fmt.Println(found, total, len(table))
}
