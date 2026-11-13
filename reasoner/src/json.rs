//! Minimal strict JSON: enough for ARC task files and our own output. No dependencies.

#[derive(Debug, Clone)]
pub enum Json { Null, Bool(bool), Num(f64), Str(String), Arr(Vec<Json>), Obj(Vec<(String, Json)>) }

impl Json {
    pub fn get(&self, key: &str) -> Option<&Json> {
        if let Json::Obj(kv) = self { kv.iter().find(|(k, _)| k == key).map(|(_, v)| v) } else { None }
    }
    pub fn arr(&self) -> Option<&Vec<Json>> { if let Json::Arr(a) = self { Some(a) } else { None } }
    pub fn num(&self) -> Option<f64> { if let Json::Num(n) = self { Some(*n) } else { None } }
    pub fn str(&self) -> Option<&str> { if let Json::Str(s) = self { Some(s) } else { None } }
}

pub fn parse(s: &str) -> Result<Json, String> {
    let b = s.as_bytes();
    let mut i = 0;
    let v = value(b, &mut i, 0)?;
    ws(b, &mut i);
    if i != b.len() { return Err("trailing data".into()); }
    Ok(v)
}

fn ws(b: &[u8], i: &mut usize) { while *i < b.len() && matches!(b[*i], b' ' | b'\n' | b'\r' | b'\t') { *i += 1; } }

fn value(b: &[u8], i: &mut usize, depth: usize) -> Result<Json, String> {
    if depth > 32 { return Err("too deep".into()); }
    ws(b, i);
    if *i >= b.len() { return Err("unexpected end".into()); }
    match b[*i] {
        b'[' => {
            *i += 1; let mut out = Vec::new(); ws(b, i);
            if *i < b.len() && b[*i] == b']' { *i += 1; return Ok(Json::Arr(out)); }
            loop {
                out.push(value(b, i, depth + 1)?); ws(b, i);
                if *i >= b.len() { return Err("unclosed array".into()); }
                match b[*i] { b',' => *i += 1, b']' => { *i += 1; return Ok(Json::Arr(out)); } _ => return Err("bad array".into()) }
            }
        }
        b'{' => {
            *i += 1; let mut out = Vec::new(); ws(b, i);
            if *i < b.len() && b[*i] == b'}' { *i += 1; return Ok(Json::Obj(out)); }
            loop {
                ws(b, i);
                let k = match value(b, i, depth + 1)? { Json::Str(s) => s, _ => return Err("bad key".into()) };
                ws(b, i);
                if *i >= b.len() || b[*i] != b':' { return Err("expected :".into()); }
                *i += 1;
                let v = value(b, i, depth + 1)?; out.push((k, v)); ws(b, i);
                if *i >= b.len() { return Err("unclosed object".into()); }
                match b[*i] { b',' => *i += 1, b'}' => { *i += 1; return Ok(Json::Obj(out)); } _ => return Err("bad object".into()) }
            }
        }
        b'"' => {
            *i += 1; let mut s = String::new();
            while *i < b.len() && b[*i] != b'"' {
                if b[*i] == b'\\' { *i += 1; if *i >= b.len() { return Err("bad escape".into()); }
                    s.push(match b[*i] { b'n' => '\n', b't' => '\t', b'r' => '\r', c => c as char }); }
                else { s.push(b[*i] as char); }
                *i += 1;
            }
            if *i >= b.len() { return Err("unclosed string".into()); }
            *i += 1; Ok(Json::Str(s))
        }
        b't' if b[*i..].starts_with(b"true") => { *i += 4; Ok(Json::Bool(true)) }
        b'f' if b[*i..].starts_with(b"false") => { *i += 5; Ok(Json::Bool(false)) }
        b'n' if b[*i..].starts_with(b"null") => { *i += 4; Ok(Json::Null) }
        _ => {
            let st = *i;
            while *i < b.len() && matches!(b[*i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') { *i += 1; }
            std::str::from_utf8(&b[st..*i]).ok().and_then(|t| t.parse::<f64>().ok()).map(Json::Num).ok_or_else(|| "bad number".into())
        }
    }
}

pub fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() { match c { '"' => o.push_str("\\\""), '\\' => o.push_str("\\\\"), '\n' => o.push_str("\\n"), c if (c as u32) < 0x20 => {}, c => o.push(c) } }
    o.push('"'); o
}
