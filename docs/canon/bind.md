# Canon: bindings

## Immutable by default

```
(let n 3)                       ; yes
(let n i64 3)                   ; no: canon.bind.redundant_type (literal fixes the type)
(let n u8 3)                    ; yes: type needed to pick u8
(let xs (vec.new <str>))        ; yes: explicit type arg when nothing else fixes it
(let xs (vec str) (vec.new))    ; no: canon.bind.type_on_let (put the type on the call)
```

## Mutation

```
(let mut n 0)
(set n (+ n 1))                 ; yes
```
There is no `+=`. Field: `(set (. p x) 1)`. Through ref: `(set (@ r) 1)`.

## Shadowing

Not allowed in the same scope (`name.dup`). Allowed in a nested scope only to change type:

```
(let s (os.arg 1))
(let n (str.to_int (& s)))      ; yes: new name for new type
```

## Unit

Never bind unit. `(let _ (io.puts "x"))` is `no: canon.bind.unit`.
