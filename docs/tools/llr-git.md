# llr-git

Read the current repo. It does not commit, push, or change files.

```
llr-git status|diff|log [--max N]
```

`--max` defaults to 20 and applies to `log` only.

Output grammar:

```
status|XY|PATH
diff|TEXT
log|HASH|SUBJECT
```

`status` is `git status --porcelain`. `XY` is the two-character code. `diff` is
`git diff --no-color HEAD`, one record per line. `log` is `git log -n N` with the
hash and subject. `|` and newline in a path, diff line, or subject are escaped.

A clean tree prints nothing and exits 0.

Errors (stderr, exit 1):

```
E|<git>|0|0|0|git.unknown|cmd=...
E|<git>|0|0|0|git.failed|cmd=...|code=N
```

A missing command is the `cli` usage error. Canon invocation: `llr-git status`.
