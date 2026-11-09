package main

import "fmt"

const w = 600
const lim = 50

func main() {
	count := 0
	for y := 0; y < w; y++ {
		for x := 0; x < w; x++ {
			cr := 2.0*float64(x)/float64(w) - 1.5
			ci := 2.0*float64(y)/float64(w) - 1.0
			zr := 0.0
			zi := 0.0
			inside := true
			for k := 0; k < lim; k++ {
				zr2 := zr*zr - zi*zi + cr
				zi = 2.0*zr*zi + ci
				zr = zr2
				if zr*zr+zi*zi > 4.0 {
					inside = false
					break
				}
			}
			if inside {
				count++
			}
		}
	}
	fmt.Println(count)
}
