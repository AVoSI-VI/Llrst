package main

import "fmt"

const n = 5000000

func main() {
	mark := make([]byte, n+1)
	for i := range mark {
		mark[i] = 1
	}
	mark[0] = 0
	mark[1] = 0
	count := 0
	last := 0
	for i := 2; i <= n; i++ {
		if mark[i] != 0 {
			count++
			last = i
			for j := i * i; j <= n; j += i {
				mark[j] = 0
			}
		}
	}
	fmt.Println(count, last)
}
