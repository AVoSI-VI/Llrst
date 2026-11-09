package main

import (
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
)

const n = 800000

type item struct {
	ID int64 `json:"id"`
}

func main() {
	parts := make([]string, n)
	for i := 0; i < n; i++ {
		parts[i] = `{"id":` + strconv.Itoa(i) + `,"ok":true}`
	}
	text := "[" + strings.Join(parts, ",") + "]"
	var values []item
	if err := json.Unmarshal([]byte(text), &values); err != nil {
		panic(err)
	}
	total := int64(0)
	for _, obj := range values {
		total += obj.ID
	}
	fmt.Println(total, len(values))
}
