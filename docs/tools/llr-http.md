# llr-http

Serves files and one exact route on a single poll loop.

```
llr-http ROOT PORT
```

Listens on 127.0.0.1 at PORT and prints `ok|port=PORT`. `GET /health` responds `ok` as `text/plain`. `GET /ws` with the WebSocket upgrade headers switches to framed messages and echoes a single text frame under 126 bytes. Any other path is a file under ROOT: one leading `/` is removed, a path containing `..` is `400`, a missing file is `404`, and a file is `200` `text/plain`. The route match is exact, including any `?query`. The process runs until it is killed.
