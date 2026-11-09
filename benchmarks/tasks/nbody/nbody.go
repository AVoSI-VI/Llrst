package main

import "fmt"

const steps = 50000
const dt = 0.01
const pi = 3.141592653589793
const days = 365.24

func nsqrt(x float64) float64 {
	if x == 0 {
		return 0
	}
	guess := 1.0
	if x > 1 {
		guess = x * 0.5
	}
	for i := 0; i < 8; i++ {
		guess = 0.5 * (guess + x/guess)
	}
	return guess
}

func advance(b [][7]float64) {
	for i := range b {
		for j := i + 1; j < len(b); j++ {
			dx := b[i][0] - b[j][0]
			dy := b[i][1] - b[j][1]
			dz := b[i][2] - b[j][2]
			dist := nsqrt(dx*dx + dy*dy + dz*dz)
			mag := dt / (dist * dist * dist)
			im := b[i][6]
			jm := b[j][6]
			b[i][3] -= dx * jm * mag
			b[i][4] -= dy * jm * mag
			b[i][5] -= dz * jm * mag
			b[j][3] += dx * im * mag
			b[j][4] += dy * im * mag
			b[j][5] += dz * im * mag
		}
	}
	for i := range b {
		b[i][0] += dt * b[i][3]
		b[i][1] += dt * b[i][4]
		b[i][2] += dt * b[i][5]
	}
}

func energy(b [][7]float64) float64 {
	e := 0.0
	for i := range b {
		e += 0.5 * b[i][6] * (b[i][3]*b[i][3] + b[i][4]*b[i][4] + b[i][5]*b[i][5])
		for j := i + 1; j < len(b); j++ {
			dx := b[i][0] - b[j][0]
			dy := b[i][1] - b[j][1]
			dz := b[i][2] - b[j][2]
			e -= b[i][6] * b[j][6] / nsqrt(dx*dx+dy*dy+dz*dz)
		}
	}
	return e
}

func main() {
	sm := 4.0 * pi * pi
	b := [][7]float64{
		{0, 0, 0, 0, 0, 0, sm},
		{4.84143144246472090, -1.16032004402742839, -0.103622044471123109, 0.00166007664274403694 * days, 0.00769901118419740425 * days, -0.0000690460016972063023 * days, 0.000954791938424326609 * sm},
		{8.34336671824457987, 4.12479856412430479, -0.403523417114321381, -0.00276742510726862411 * days, 0.00499852801234917238 * days, 0.0000230417297573763929 * days, 0.000285885980666130812 * sm},
		{12.8943695621391310, -15.1111514016986312, -0.223307578892655734, 0.00296460137564761618 * days, 0.00237847173959480950 * days, -0.0000296589568540237556 * days, 0.0000436624404335156298 * sm},
		{15.3796971148509165, -25.9193146099879641, 0.179258772950371181, 0.00268067772490389322 * days, 0.00162824170038242295 * days, -0.0000951592254519715870 * days, 0.0000515138902046611451 * sm},
	}
	px, py, pz := 0.0, 0.0, 0.0
	for i := range b {
		px += b[i][3] * b[i][6]
		py += b[i][4] * b[i][6]
		pz += b[i][5] * b[i][6]
	}
	b[0][3] = -px / sm
	b[0][4] = -py / sm
	b[0][5] = -pz / sm
	for s := 0; s < steps; s++ {
		advance(b)
	}
	fmt.Println(int64(energy(b) * 10000.0))
}
