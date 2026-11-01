/* Browser host. Canvas blit and the 64-byte event record. No layout or paint. */
(function () {
  const RING = 64;
  const q = [];
  const files = new Map();
  const fds = new Map();
  let nextFd = 4;
  let memory = null;
  let exports = null;
  let appPtr = 0;
  let stopped = false;
  let pending = false;
  let raf = 0;
  let clip = "";
  let hist = 1;
  let suppress = false;
  const canvas = document.getElementById("llr");
  const ctx = canvas.getContext("2d", { alpha: true });
  const input = document.createElement("input");
  input.id = "llr-ime";
  input.autocomplete = "off";
  input.style.position = "fixed";
  input.style.opacity = "0";
  input.style.padding = "0";
  input.style.border = "0";
  input.style.margin = "0";
  document.body.appendChild(input);
  canvas.tabIndex = 0;

  function mem() { return memory.buffer; }
  function view() { return new DataView(mem()); }
  function dec(ptr, n) { return new TextDecoder().decode(new Uint8Array(mem(), ptr, n)); }

  function blank() { return new Uint8Array(64); }

  function setI32(b, off, n) { new DataView(b.buffer).setInt32(off, n | 0, true); }

  function setF32(b, off, n) { new DataView(b.buffer).setFloat32(off, n, true); }

  function i32at(b, off) { return new DataView(b.buffer).getInt32(off, true); }

  function push(ev) {
    const n = q.length;
    if (n > 0 && i32at(ev, 0) === 1 && i32at(ev, 4) === 1) {
      const prev = q[n - 1];
      if (i32at(prev, 0) === 1 && i32at(prev, 4) === 1) {
        q[n - 1] = ev;
        return;
      }
    }
    if (q.length >= RING) q.shift();
    q.push(ev);
  }

  function pushKind(kind) {
    const ev = blank();
    setI32(ev, 0, kind);
    push(ev);
  }

  function ptrEv(kind, x, y, z, w) {
    const ev = blank();
    setI32(ev, 0, 1);
    setI32(ev, 4, kind);
    setF32(ev, 16, x);
    setF32(ev, 20, y);
    setF32(ev, 24, z);
    setF32(ev, 28, w);
    push(ev);
  }

  function xy(e) {
    const r = canvas.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  }

  function mods(e) {
    return (e.shiftKey ? 1 : 0) | (e.ctrlKey ? 2 : 0) | (e.altKey ? 4 : 0) | (e.metaKey ? 8 : 0);
  }

  function codeOf(e) {
    if (e.key === "Tab") return 9;
    if (e.key === "Enter") return 13;
    if (e.key === "Backspace") return 8;
    if (e.key === "Escape") return 27;
    if (e.key.length === 1) return e.key.codePointAt(0);
    return 0;
  }

  function keyEv(down, e) {
    const ev = blank();
    setI32(ev, 0, 2);
    setI32(ev, 4, down);
    setI32(ev, 8, codeOf(e));
    setI32(ev, 12, mods(e));
    push(ev);
  }

  function pushText(s) {
    const enc = new TextEncoder().encode(s);
    let off = 0;
    while (off < enc.length) {
      let m = Math.min(32, enc.length - off);
      while (m > 1 && (enc[off + m - 1] & 0xc0) === 0x80) m -= 1;
      const ev = blank();
      setI32(ev, 0, 3);
      setI32(ev, 4, m);
      ev.set(enc.subarray(off, off + m), 32);
      push(ev);
      off += m;
    }
  }

  function logical() {
    const r = canvas.getBoundingClientRect();
    return {
      w: r.width || window.innerWidth || 800,
      h: r.height || window.innerHeight || 600,
    };
  }

  function unpremul(c, a) {
    if (a === 0 || a === 255) return c;
    return Math.min(255, Math.round((c * 255) / a));
  }

  function present(ptr, w, h) {
    if (w < 1 || h < 1) return;
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
    }
    const src = new Uint8Array(mem(), ptr, w * h * 4);
    const img = ctx.createImageData(w, h);
    const dst = img.data;
    for (let i = 0; i < w * h; i += 1) {
      const o = i * 4;
      const a = src[o + 3];
      dst[o] = unpremul(src[o + 2], a);
      dst[o + 1] = unpremul(src[o + 1], a);
      dst[o + 2] = unpremul(src[o], a);
      dst[o + 3] = a;
    }
    ctx.putImageData(img, 0, 0);
  }

  function size(wp, hp) {
    const box = logical();
    const v = view();
    v.setInt32(wp, box.w | 0, true);
    v.setInt32(hp, box.h | 0, true);
  }

  function scale() { return window.devicePixelRatio || 1; }

  function now_ms() { return BigInt(Math.trunc(performance.now())); }

  function request_frame() {
    if (pending || stopped) return;
    pending = true;
    raf = requestAnimationFrame(() => {
      pending = false;
      if (!stopped && exports) exports.llr_frame();
    });
  }

  function poll_event(ptr) {
    if (q.length === 0) return 0;
    new Uint8Array(mem()).set(q.shift(), ptr);
    return 1;
  }

  function set_cursor(id) {
    const names = ["default", "text", "pointer", "crosshair", "wait"];
    canvas.style.cursor = names[id] || "default";
  }

  function clipboard_get(buf, cap) {
    const enc = new TextEncoder().encode(clip);
    if (cap <= 0) return enc.length;
    const n = Math.min(cap, enc.length);
    if (n > 0) new Uint8Array(mem()).set(enc.subarray(0, n), buf);
    return n;
  }

  function clipboard_set(buf, n) {
    clip = dec(buf, n);
    if (navigator.clipboard && navigator.clipboard.writeText) {
      navigator.clipboard.writeText(clip).catch(() => {});
    }
  }

  function text_input(on, x, y, w, h) {
    if (!on) {
      input.blur();
      input.style.display = "none";
      return;
    }
    input.style.display = "block";
    input.style.left = x + "px";
    input.style.top = y + "px";
    input.style.width = Math.max(w, 1) + "px";
    input.style.height = Math.max(h, 1) + "px";
    input.focus();
  }

  function history(n) {
    const next = n | 0;
    if (next === hist) return;
    if (next > hist) {
      suppress = false;
      window.history.pushState({ n: next }, "");
      hist = next;
      return;
    }
    if (suppress) {
      suppress = false;
      hist = next;
      return;
    }
    hist = next;
    window.history.back();
  }

  function quit() {
    stopped = true;
    if (raf) cancelAnimationFrame(raf);
    input.blur();
  }

  function bind(p) { appPtr = p; }
  function app() { return appPtr; }

  function fd_write(fd, iovs, n, out) {
    const v = view();
    let total = 0;
    let text = "";
    for (let i = 0; i < n; i += 1) {
      const base = iovs + i * 8;
      const p = v.getInt32(base, true);
      const len = v.getInt32(base + 4, true);
      text += dec(p, len);
      total += len;
    }
    v.setInt32(out, total, true);
    if (fd === 2) console.error(text);
    else console.log(text);
    return 0;
  }

  function fd_read(fd, iovs, n, out) {
    const f = fds.get(fd);
    if (!f) return 8;
    let got = 0;
    const v = view();
    for (let i = 0; i < n; i += 1) {
      const base = iovs + i * 8;
      const p = v.getInt32(base, true);
      const len = v.getInt32(base + 4, true);
      const m = Math.max(0, Math.min(len, f.data.length - f.pos));
      if (m > 0) {
        new Uint8Array(mem()).set(f.data.subarray(f.pos, f.pos + m), p);
        f.pos += m;
        got += m;
      }
    }
    v.setInt32(out, got, true);
    return 0;
  }

  function path_open(_fd, _dir, path, len, _o, _base, _inh, _flags, out) {
    const name = dec(path, len);
    const data = files.get(name) || files.get(name.replace(/^\//, ""));
    if (!data) return 44;
    const id = nextFd;
    nextFd += 1;
    fds.set(id, { data, pos: 0 });
    view().setInt32(out, id, true);
    return 0;
  }

  function fd_close(fd) {
    fds.delete(fd);
    return 0;
  }

  function clock_time_get(_id, _prec, out) {
    view().setBigInt64(out, BigInt(Math.trunc(performance.now() * 1e6)), true);
    return 0;
  }

  function proc_exit(code) {
    quit();
    throw new Error("proc_exit " + code);
  }

  function sizes(a, b) {
    const v = view();
    v.setInt32(a, 0, true);
    v.setInt32(b, 0, true);
    return 0;
  }

  function random_get(buf, len) {
    crypto.getRandomValues(new Uint8Array(mem(), buf, len));
    return 0;
  }

  const llr = {
    fd_write, fd_read, path_open, fd_close, clock_time_get, proc_exit,
    args_get: () => 0,
    args_sizes_get: sizes,
    environ_get: () => 0,
    environ_sizes_get: sizes,
    random_get,
    present, size, scale, now_ms, request_frame, poll_event, set_cursor,
    clipboard_get, clipboard_set, text_input, quit, bind, app, history,
  };

  canvas.addEventListener("pointerdown", (e) => {
    canvas.setPointerCapture(e.pointerId);
    const p = xy(e);
    ptrEv(0, p.x, p.y, 0, 0);
  });
  canvas.addEventListener("pointermove", (e) => {
    const p = xy(e);
    ptrEv(1, p.x, p.y, 0, 0);
  });
  canvas.addEventListener("pointerup", (e) => {
    const p = xy(e);
    ptrEv(2, p.x, p.y, 0, 0);
  });
  canvas.addEventListener("pointercancel", (e) => {
    const p = xy(e);
    ptrEv(3, p.x, p.y, 0, 0);
  });
  canvas.addEventListener("wheel", (e) => {
    e.preventDefault();
    const p = xy(e);
    ptrEv(4, p.x, p.y, e.deltaX, e.deltaY);
  }, { passive: false });
  window.addEventListener("keydown", (e) => {
    if (e.key === "Tab") e.preventDefault();
    keyEv(1, e);
  });
  window.addEventListener("keyup", (e) => keyEv(0, e));
  input.addEventListener("input", () => {
    if (input.value) pushText(input.value);
    input.value = "";
  });
  window.addEventListener("paste", (e) => {
    const t = e.clipboardData && e.clipboardData.getData("text/plain");
    if (t) clip = t;
  });
  window.addEventListener("popstate", () => {
    suppress = true;
    pushKind(8);
  });
  window.addEventListener("resize", () => pushKind(4));
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible") pushKind(4);
  });

  async function boot() {
    const fontUrl = "ui/fonts/LiberationSans-Regular.ttf";
    const wasm = await fetch("app.wasm").then((r) => r.arrayBuffer());
    const fontRes = await fetch(fontUrl);
    if (fontRes.ok) files.set(fontUrl, new Uint8Array(await fontRes.arrayBuffer()));
    const inst = await WebAssembly.instantiate(wasm, { llr });
    exports = inst.instance.exports;
    memory = exports.memory;
    exports._start();
  }

  boot().catch((e) => console.error(e));
})();
