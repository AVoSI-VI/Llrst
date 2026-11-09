package main

import (
	"fmt"
	"strconv"
	"strings"
)

const n = 800000

func main() {
	parts := make([]string, n)
	for i := 0; i < n; i++ {
		parts[i] = strconv.Itoa(i)
	}
	text := strings.Join(parts, ",")
	acc := int64(0)
	for i := 0; i < len(text); i++ {
		acc = (acc*131 + int64(text[i])) & 0xffffffff
	}
	fmt.Println(len(text), acc)
}
