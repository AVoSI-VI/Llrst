package main

import "fmt"

const n = 500
const iters = 10

func aval(i, j int) float64 {
	ij := i + j
	return 1.0 / float64((ij*(ij+1))/2+i+1)
}

func mul(v []float64, trans bool) []float64 {
	out := make([]float64, n)
	for i := 0; i < n; i++ {
		s := 0.0
		for j := 0; j < n; j++ {
			a := aval(i, j)
			if trans {
				a = aval(j, i)
			}
			s += a * v[j]
		}
		out[i] = s
	}
	return out
}

func main() {
	u := make([]float64, n)
	for i := range u {
		u[i] = 1
	}
	for k := 0; k < iters; k++ {
		v := mul(u, false)
		u = mul(v, true)
	}
	acc := int64(0)
	for _, x := range u {
		acc += int64(x * 1000000.0)
	}
	fmt.Println(acc)
}
