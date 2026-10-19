# llr-write

Create or replace a whole file. The body is stdin.

```
llr-write PATH
```

Output grammar:

```
ok|PATH|bytes=N
```

An empty stdin still creates an empty file (`bytes=0`). An existing file is truncated.
Parent directories are not created.

Errors (stderr, exit 1):

```
E|<write>|0|0|0|io.write|path=...
```

A missing `PATH` is the `cli` usage error. Canon invocation: `llr-write PATH` with the
file body on stdin.
