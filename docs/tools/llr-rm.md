# llr-rm

Delete one file, or a directory and everything under it.

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

A missing path produces `io.remove`. A missing `PATH` argument is the `cli` usage error.
Canon invocation: `llr-rm PATH`.
