//! Lexer and S-expression reader. The whole surface grammar is here.
use crate::diag::{Diag, PResult, Span};

#[derive(Clone, Debug)]
pub enum Sexpr {
    Atom(String, Span),
    Str(Vec<u8>, Span),
    List(Vec<Sexpr>, Span),
}

impl Sexpr {
    pub fn span(&self) -> Span {
        match self {
            Sexpr::Atom(_, s) | Sexpr::Str(_, s) | Sexpr::List(_, s) => *s,
        }
    }
    pub fn atom(&self) -> Option<&str> {
        match self {
            Sexpr::Atom(a, _) => Some(a),
            _ => None,
        }
    }
    pub fn list(&self) -> Option<&[Sexpr]> {
        match self {
            Sexpr::List(l, _) => Some(l),
            _ => None,
        }
    }
    pub fn is_atom(&self, s: &str) -> bool {
        self.atom() == Some(s)
    }
    /// Head atom of a list, if any.
    pub fn head(&self) -> Option<&str> {
        self.list().and_then(|l| l.first()).and_then(|h| h.atom())
    }
    /// Compact rendering used in `fix=` and `form=` diagnostics.
    pub fn show(&self) -> String {
        match self {
            Sexpr::Atom(a, _) => a.clone(),
            Sexpr::Str(s, _) => format!("{:?}", String::from_utf8_lossy(s)),
            Sexpr::List(l, _) => {
                let parts: Vec<String> = l.iter().map(|x| x.show()).collect();
                format!("({})", parts.join(" "))
            }
        }
    }
}

pub fn read_file(file: u32, text: &str) -> PResult<Vec<Sexpr>> {
    let mut r = Reader { file, b: text.as_bytes(), i: 0 };
    let mut out = Vec::new();
    loop {
        r.skip_ws();
        if r.i >= r.b.len() {
            return Ok(out);
        }
        if r.b[r.i] == b')' {
            return Err(Diag::error(Span::new(file, r.i, r.i + 1), "parse.unbalanced").kv("got", ")"));
        }
        out.push(r.form()?);
    }
}

struct Reader<'a> {
    file: u32,
    b: &'a [u8],
    i: usize,
}

impl<'a> Reader<'a> {
    fn skip_ws(&mut self) {
        while self.i < self.b.len() {
            match self.b[self.i] {
                b' ' | b'\t' | b'\n' | b'\r' => self.i += 1,
                b';' => {
                    while self.i < self.b.len() && self.b[self.i] != b'\n' {
                        self.i += 1;
                    }
                }
                _ => return,
            }
        }
    }

    fn form(&mut self) -> PResult<Sexpr> {
        let start = self.i;
        match self.b[self.i] {
            b'(' => {
                self.i += 1;
                let mut items = Vec::new();
                loop {
                    self.skip_ws();
                    if self.i >= self.b.len() {
                        return Err(Diag::error(Span::new(self.file, start, start + 1), "parse.unbalanced")
                            .kv("want", ")"));
                    }
                    if self.b[self.i] == b')' {
                        self.i += 1;
                        return Ok(Sexpr::List(items, Span::new(self.file, start, self.i)));
                    }
                    items.push(self.form()?);
                }
            }
            b'"' => self.string(),
            _ => {
                while self.i < self.b.len() && !is_delim(self.b[self.i]) {
                    self.i += 1;
                }
                let text = std::str::from_utf8(&self.b[start..self.i]).unwrap().to_string();
                Ok(Sexpr::Atom(text, Span::new(self.file, start, self.i)))
            }
        }
    }

    fn string(&mut self) -> PResult<Sexpr> {
        let start = self.i;
        self.i += 1;
        let mut out: Vec<u8> = Vec::new();
        loop {
            if self.i >= self.b.len() {
                return Err(Diag::error(Span::new(self.file, start, self.i), "lex.unterminated_string"));
            }
            let c = self.b[self.i];
            self.i += 1;
            match c {
                b'"' => break,
                b'\\' => {
                    let e = *self.b.get(self.i).ok_or_else(|| {
                        Diag::error(Span::new(self.file, start, self.i), "lex.unterminated_string")
                    })?;
                    self.i += 1;
                    match e {
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'0' => out.push(0),
                        b'"' => out.push(b'"'),
                        b'\\' => out.push(b'\\'),
                        b'x' => {
                            let h = self.b.get(self.i..self.i + 2).and_then(|h| std::str::from_utf8(h).ok());
                            let v = h.and_then(|h| u8::from_str_radix(h, 16).ok());
                            match v {
                                Some(v) => {
                                    out.push(v);
                                    self.i += 2;
                                }
                                None => {
                                    return Err(Diag::error(Span::new(self.file, self.i - 2, self.i), "lex.bad_escape"))
                                }
                            }
                        }
                        b'u' => {
                            let cp = self.unicode_escape()?;
                            let mut buf = [0u8; 4];
                            let ch = char::from_u32(cp).ok_or_else(|| Diag::error(Span::new(self.file, self.i - 2, self.i), "lex.bad_escape"))?;
                            out.extend(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        _ => return Err(Diag::error(Span::new(self.file, self.i - 2, self.i), "lex.bad_escape")),
                    }
                }
                _ => out.push(c),
            }
        }
        Ok(Sexpr::Str(out, Span::new(self.file, start, self.i)))
    }

    /// `\u{HHHH}` with 1 to 6 hex digits, at most U+10FFFF, not a surrogate.
    fn unicode_escape(&mut self) -> PResult<u32> {
        let start = self.i;
        if self.b.get(self.i) != Some(&b'{') {
            return Err(Diag::error(Span::new(self.file, start - 2, self.i), "lex.bad_escape"));
        }
        self.i += 1;
        let hex = self.i;
        while self.b.get(self.i).map_or(false, |c| c.is_ascii_hexdigit()) {
            self.i += 1;
        }
        let n = self.i - hex;
        if n == 0 || n > 6 || self.b.get(self.i) != Some(&b'}') {
            return Err(Diag::error(Span::new(self.file, start - 2, self.i), "lex.bad_escape"));
        }
        let text = std::str::from_utf8(&self.b[hex..self.i]).unwrap();
        self.i += 1;
        let cp = u32::from_str_radix(text, 16).unwrap();
        if cp > 0x10FFFF || (0xD800..=0xDFFF).contains(&cp) {
            return Err(Diag::error(Span::new(self.file, start - 2, self.i), "lex.bad_escape"));
        }
        Ok(cp)
    }
}

fn is_delim(c: u8) -> bool {
    matches!(c, b'(' | b')' | b'"' | b' ' | b'\t' | b'\n' | b'\r' | b';')
}
