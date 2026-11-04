# Canon: strings

`str` is bytes. Indexing is by byte. There is no char type; non-ASCII is passed through.

| task | form |
|------|------|
| literal | `"text"` has type `(& str)` |
| own a copy | `(str.from "text")` or `(str.clone (& s))` |
| empty owned | `(str.new)` |
| length | `(str.len (& s))` |
| byte at | `(str.at (& s) i)` returns `u8`, panics on oob |
| concat | `(str.cat (& a) (& b))` returns new `str` |
| append in place | `(str.push (&mut s) (& t))`, `(str.push_byte (&mut s) 10u8)` |
| compare | `(str.eq (& a) (& b))` |
| substring | `(str.slice (& s) lo hi)` new `str`, `[lo hi)` |
| find | `(str.find (& s) (& needle))` returns `(opt i64)` |
| prefix / suffix | `(str.starts (& s) (& p))` `(str.ends (& s) (& p))` |
| split | `(str.split (& s) 124u8)` returns `(vec str)` (byte separator) |
| contains | `(str.has (& s) (& needle))`, `(str.has_byte (& s) 10u8)` |
| compare order | `(str.cmp (& a) (& b))` negative, 0, positive |
| count | `(str.count (& s) (& needle))` |
| lines | `(str.lines (& s))` returns `(vec str)`, strips `\n` and `\r\n` |
| trim | `(str.trim (& s))` |
| int to str | `(str.from_int n)` |
| str to int | `(str.to_int (& s))` returns `(opt i64)` |
| replace | `(str.replace (& s) (& old) (& new))` all occurrences |
| join | `(str.join (& parts) (& sep))` |
| lower | `(str.lower (& s))` ASCII only |
| upper | `(str.upper (& s))` ASCII only |
| trim start / end | `(str.trim_start (& s))` `(str.trim_end (& s))` |
| repeat | `(str.repeat (& s) n)` |
| last match | `(str.rfind (& s) (& needle))` `(str.rfind_byte (& s) 46u8)` returns `(opt i64)` |
| split on text | `(str.split_str (& s) (& sep))` returns `(vec str)`; an empty separator is one part |
| replace one byte | `(str.replace_byte (& s) 92u8 47u8)` |
| pad on the right | `(str.pad_right (& s) width 32u8)` no extra bytes when `s` is already long enough |
| bytes | `(str.to_bytes (& s))` `(str.from_bytes (& v))` |
| character class | `(str.is_space b)` `(str.is_digit b)` `(str.is_alpha b)` `(str.is_alnum b)` ASCII |
| UTF-8 decode | `(str.utf8_at (& s) i)` returns `uch` with `cp` and `n`; an invalid byte is U+FFFD, length 1 |
| UTF-8 encode | `(str.push_utf8 (&mut s) cp)` appends a scalar and returns `false` for a surrogate or a value above U+10FFFF |
| UTF-8 check | `(str.utf8_ok (& s))` rejects overlongs, surrogates, and codepoints above U+10FFFF |
| radix | `(str.from_int_radix n 16)` `(str.to_int_radix (& s) 16)` radix 2 through 36, lowercase; `(str.to_int (& s))` stays decimal |
| float | `(str.from_f64 x digits)` fixed fraction, truncated toward zero; non-finite is `nan`, `inf`, `-inf`. `(str.to_f64 (& s))` returns `(opt f64)` |
| C string | `(str.cstr (& s))` NUL-terminated copy for externs; `(str.from_cstr p)` back |

## Building output lines

```
(let mut o (str.new))
(str.push (&mut o) (& path))
(str.push_byte (&mut o) 124u8)
(str.push (&mut o) (& (str.from_int n)))
(io.puts (& o))
```
For 3+ parts use `fmt`: `(io.puts (& (fmt.pipe3 (& path) (& (str.from_int n)) (& text))))`.

## Records, hex, and padding

| task | form |
|------|------|
| hex | `(fmt.hex n width)` lowercase, no prefix, minimum width; a negative value keeps the minus sign |
| fixed float | `(fmt.float x digits)` same truncation as `str.from_f64` |
| pad right | `(fmt.pad_right (& s) width fill)` |
| read a record | `(fmt.fields (& s))` splits on `\|` and undoes `fmt.field` (`\x7c` and `\n`) |
| field as int | `(fmt.field_int (& s))` returns `(opt i64)` |

## Scanning text

`(use std.scan)`. `cur` borrows the string and holds the index.

| task | form |
|------|------|
| cursor | `(cur (& s) 0)` |
| byte / index | `(scan.peek (& c))` `(scan.at (& c))`; past the end, peek is `0u8` |
| whitespace | `(scan.skip_ws (&mut c))` |
| name | `(scan.is_name b)` is an ASCII letter, digit, or `_`. `(scan.read_name (&mut c))` skips whitespace, then `[A-Za-z_][A-Za-z0-9_]*`, or returns empty |
| hex digit | `(scan.hex_dig b)` returns `(opt i64)` |
| one byte | `(scan.expect (&mut c) 40u8)` consumes it and returns `bool` |

## Numbers

`(use std.math)` and `(use std.math_trig)`. `f32` entry points are the `32` twins and round an `f64` result. `math.approx` uses an absolute epsilon of `0.000001`.

| task | form |
|------|------|
| integer min / max / clamp / abs | `(math.imin a b)` `(math.imax a b)` `(math.iclamp x lo hi)` `(math.iabs n)`; `iabs` of the minimum `i64` stays that minimum |
| integer power | `(math.ipow x n)`; a negative exponent is `0`, and `(math.ipow 0 0)` is `1` |
| align | `(math.align_up x a)`; `a <= 1` returns `x` |
| thousandths | `(math.milli x)` truncates `(as i64 (* (as f64 x) 1000.0))` |
| non-finite | `(math.nan)` `(math.inf)` `(math.pow2 e)` |
| float clamp / mix / remainder | `(math.clamp x lo hi)` `(math.lerp a b t)` `(math.fmod x y)`; a zero divisor is `nan` |
| exp / ln | `(math_trig.exp x)` `(math_trig.ln x)` |
| trig and logs | `(math_trig.tan x)` `(math_trig.asin x)` `(math_trig.acos x)` `(math_trig.log2 x)` `(math_trig.log10 x)` |
