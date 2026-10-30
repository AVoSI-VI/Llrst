# llr-http

Serves files and exact routes on a single poll loop.

```
llr-http ROOT PORT
```

Listens on 127.0.0.1 at PORT and prints `ok|port=PORT`. The registered routes are exact, including any `?query`:

- `GET /health` responds `ok` as `text/plain`.
- `POST /graphql` takes the query text as the body and responds with JSON.
- `POST /soap` takes one XML envelope and responds with XML.

A `GET` that carries `Upgrade: websocket` and `Connection: upgrade` is switched to framed messages by `http.serve` on any path, not as a registered route. `/ws` is the usual example. A single masked text frame under 126 bytes is echoed.

Any other path is a file under ROOT. One leading `/` is removed, a path containing `..` is `400`, a missing file is `404`, and a file is `200` `text/plain`. The process runs until it is killed.
