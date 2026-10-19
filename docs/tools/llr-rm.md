# llr-rm

Delete one file. A directory is left in place.

```
llr-rm PATH
```

Output grammar:

```
ok|PATH
```

Errors (stderr, exit 1):

```
E|<rm>|0|0|0|io.remove|path=...
```

A missing path and a directory both produce `io.remove`. A missing `PATH` argument is
the `cli` usage error. Canon invocation: `llr-rm PATH`.
