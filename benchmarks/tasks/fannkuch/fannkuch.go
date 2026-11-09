package main

import "fmt"

const n = 9

func flips(perm []int64) int64 {
	q := append([]int64(nil), perm...)
	count := int64(0)
	for q[0] != 0 {
		k := q[0]
		i := int64(0)
		j := k
		for i < j {
			q[i], q[j] = q[j], q[i]
			i++
			j--
		}
		count++
	}
	return count
}

var perm []int64
var best, checksum, sign int64

func rec(k int) {
	if k == 1 {
		if perm[0] != 0 {
			got := flips(perm)
			checksum += sign * got
			if got > best {
				best = got
			}
		}
		sign = -sign
		return
	}
	rec(k - 1)
	for i := 0; i < k-1; i++ {
		t := perm[0]
		for j := 0; j < k-1; j++ {
			perm[j] = perm[j+1]
		}
		perm[k-1] = t
		rec(k - 1)
	}
}

func main() {
	perm = make([]int64, n)
	for i := 0; i < n; i++ {
		perm[i] = int64(i)
	}
	best = 0
	checksum = 0
	sign = 1
	rec(n)
	fmt.Println(checksum, best)
}
