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
| split | `(str.split (& s) 124u8)` returns `(vec str)` |
| lines | `(str.lines (& s))` returns `(vec str)`, strips `\n` and `\r\n` |
| trim | `(str.trim (& s))` |
| int to str | `(str.from_int n)` |
| str to int | `(str.to_int (& s))` returns `(opt i64)` |
| replace | `(str.replace (& s) (& old) (& new))` all occurrences |
| join | `(str.join (& parts) (& sep))` |
| lower | `(str.lower (& s))` ASCII only |

## Building output lines

```
(let mut o (str.new))
(str.push (&mut o) (& path))
(str.push_byte (&mut o) 124u8)
(str.push (&mut o) (& (str.from_int n)))
(io.puts (& o))
```
For 3+ parts use `fmt`: `(io.puts (& (fmt.pipe3 (& path) (& (str.from_int n)) (& text))))`.
