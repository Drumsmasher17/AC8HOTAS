//! Restricted Lua literals with byte spans. Never evaluates Lua or rewrites unrelated text.
use crate::{actions::ACTIONS, model::Binding};
use std::{collections::BTreeSet, ops::Range};

#[derive(Clone, Debug)]
enum Kind {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Table(Vec<Field>, usize, bool),
}
#[derive(Clone, Debug)]
struct Value {
    kind: Kind,
    span: Range<usize>,
}
#[derive(Clone, Debug)]
struct Field {
    key: String,
    value: Value,
}
#[derive(Clone, Debug)]
pub struct Document {
    pub text: String,
    pub bindings: Vec<Option<Binding>>,
    table: Value,
}

pub fn valid_guid(s: &str) -> bool {
    s.len() == 38
        && s.starts_with('{')
        && s.ends_with('}')
        && s.bytes().enumerate().all(|(i, c)| match i {
            0 => c == b'{',
            37 => c == b'}',
            9 | 14 | 19 | 24 => c == b'-',
            _ => c.is_ascii_hexdigit(),
        })
}
pub fn axis_token(s: &str) -> bool {
    matches!(
        s,
        "X" | "Y" | "Z" | "Rx" | "Ry" | "Rz" | "Slider1" | "Slider2"
    )
}
pub fn digital_token(s: &str) -> bool {
    matches!(s, "POV1_Up" | "POV1_Down" | "POV1_Left" | "POV1_Right")
        || s.strip_prefix("Button").is_some_and(|n| {
            !n.starts_with('0') && n.parse::<u16>().is_ok_and(|v| (1..=128).contains(&v))
        })
}
pub fn validate_binding(index: usize, b: &Binding) -> Result<(), String> {
    if !valid_guid(&b.device) {
        return Err(format!("{}: invalid device GUID", ACTIONS[index].label));
    }
    if ACTIONS[index].axis && !axis_token(&b.input) {
        return Err(format!("{} needs an axis", ACTIONS[index].label));
    }
    if !ACTIONS[index].axis && (!digital_token(&b.input) || b.invert || b.zero_to_one) {
        return Err(format!(
            "{} needs a button or first-hat direction",
            ACTIONS[index].label
        ));
    }
    Ok(())
}
fn fields(v: &Value) -> Result<&[Field], String> {
    if let Kind::Table(f, _, _) = &v.kind {
        Ok(f)
    } else {
        Err("Expected a table".into())
    }
}
fn field<'a>(f: &'a [Field], k: &str) -> Option<&'a Value> {
    f.iter().find(|x| x.key == k).map(|x| &x.value)
}
fn string(v: &Value) -> Result<&str, String> {
    if let Kind::String(s) = &v.kind {
        Ok(s)
    } else {
        Err("Expected a string".into())
    }
}
fn boolean(v: &Value) -> Result<bool, String> {
    if let Kind::Bool(b) = v.kind {
        Ok(b)
    } else {
        Err("Expected true or false".into())
    }
}
fn locate(text: &str, at: usize, message: impl std::fmt::Display) -> String {
    let prefix = &text[..at.min(text.len())];
    let line = prefix.bytes().filter(|c| *c == b'\n').count() + 1;
    let col = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
    format!("Line {line}, column {col}: {message}")
}

impl Document {
    pub fn parse(text: String) -> Result<Self, String> {
        if text.len() > 1024 * 1024 {
            return Err("Configuration exceeds 1 MiB".into());
        }
        let mut p = Parser {
            src: &text,
            pos: if text.starts_with('\u{feff}') { 3 } else { 0 },
            depth: 0,
        };
        p.space()?;
        if p.ident()? != "return" {
            return Err(locate(&text, 0, "Expected return { ... }"));
        }
        let root = p.value()?;
        p.space()?;
        if p.peek() == Some(b';') {
            p.pos += 1;
            p.space()?;
        }
        if p.pos != text.len() {
            return Err(locate(&text, p.pos, "Unexpected code after configuration"));
        }
        let top = fields(&root)?;
        for f in top {
            if !matches!(f.key.as_str(), "version" | "bindings" | "trace_throttle") {
                return Err(locate(
                    &text,
                    f.value.span.start,
                    format!("Unknown config field: {}", f.key),
                ));
            }
        }
        if !matches!(field(top,"version").map(|v|&v.kind),Some(Kind::Number(n)) if *n==1.0) {
            return Err("Expected config version = 1".into());
        }
        let table = field(top, "bindings")
            .ok_or("Missing bindings table")?
            .clone();
        let mut bindings = vec![None; ACTIONS.len()];
        for f in fields(&table)? {
            let index = ACTIONS.iter().position(|a| a.key == f.key).ok_or_else(|| {
                locate(
                    &text,
                    f.value.span.start,
                    format!("Unknown action: {}", f.key),
                )
            })?;
            if matches!(f.value.kind, Kind::Nil) {
                continue;
            }
            let bf = fields(&f.value).map_err(|e| locate(&text, f.value.span.start, e))?;
            for entry in bf {
                if !matches!(entry.key.as_str(), "device" | "input" | "invert" | "mode") {
                    return Err(locate(
                        &text,
                        entry.value.span.start,
                        format!("Unknown binding field: {}", entry.key),
                    ));
                }
            }
            let get = |k: &str| field(bf, k).ok_or_else(|| format!("{}: missing {k}", f.key));
            let device = string(get("device")?)?.to_ascii_uppercase();
            let input = string(get("input")?)?.to_owned();
            let invert = field(bf, "invert")
                .map(boolean)
                .transpose()?
                .unwrap_or(false);
            let mode = field(bf, "mode").map(string).transpose()?;
            let zero_to_one = match mode {
                None => f.key == "Throttle",
                Some("zero_to_one" | "range") => true,
                Some("minus_one_to_one" | "signed") => false,
                _ => return Err(format!("{}: unknown axis mode", f.key)),
            };
            if !ACTIONS[index].axis && mode.is_some() {
                return Err(format!("{}: buttons/hats do not use mode", f.key));
            }
            let b = Binding {
                device,
                input,
                invert,
                zero_to_one,
            };
            validate_binding(index, &b).map_err(|e| locate(&text, f.value.span.start, e))?;
            bindings[index] = Some(b);
        }
        Ok(Self {
            text,
            bindings,
            table,
        })
    }

    pub fn render(&self, draft: &[Option<Binding>]) -> Result<String, String> {
        if draft.len() != ACTIONS.len() {
            return Err("Invalid action count".into());
        }
        let mut edits: Vec<(Range<usize>, String)> = Vec::new();
        let entries = fields(&self.table)?;
        let mut missing = Vec::new();
        for (i, b) in draft.iter().enumerate() {
            if let Some(b) = b {
                validate_binding(i, b)?;
            }
            if *b == self.bindings[i] {
                continue;
            }
            if let Some(original) = entries.iter().find(|f| f.key == ACTIONS[i].key) {
                match (&original.value.kind, b) {
                    (Kind::Table(bf, close, trailing), Some(b)) => {
                        let desired = [
                            ("device", quote(&b.device)),
                            ("input", quote(&b.input)),
                            ("invert", b.invert.to_string()),
                            (
                                "mode",
                                quote(if b.zero_to_one {
                                    "zero_to_one"
                                } else {
                                    "minus_one_to_one"
                                }),
                            ),
                        ];
                        let old = self.bindings[i].as_ref().expect("parsed binding");
                        let changed = [
                            b.device != old.device,
                            b.input != old.input,
                            b.invert != old.invert,
                            b.zero_to_one != old.zero_to_one,
                        ];
                        let mut added = Vec::new();
                        for (n, (key, value)) in desired.into_iter().enumerate() {
                            if !changed[n] {
                                continue;
                            }
                            if let Some(v) = field(bf, key) {
                                edits.push((v.span.clone(), value));
                            } else {
                                added.push(format!("{key} = {value}"));
                            }
                        }
                        insert_fields(&mut edits, bf, *close, *trailing, &added, "    ");
                    }
                    (_, b) => {
                        edits.push((original.value.span.clone(), binding_literal(i, b.as_ref())))
                    }
                }
            } else if b.is_some() {
                missing.push(format!(
                    "{} = {}",
                    ACTIONS[i].key,
                    binding_literal(i, b.as_ref())
                ));
            }
        }
        if let Kind::Table(_, close, trailing) = &self.table.kind {
            insert_fields(&mut edits, entries, *close, *trailing, &missing, "        ");
        }
        edits.sort_by_key(|a| std::cmp::Reverse(a.0.start));
        let mut result = self.text.clone();
        for (range, value) in edits {
            result.replace_range(range, &value);
        }
        let check = Self::parse(result.clone())?;
        if check.bindings != draft {
            return Err("Serialized configuration failed readback".into());
        }
        Ok(result)
    }
}
fn insert_fields(
    edits: &mut Vec<(Range<usize>, String)>,
    f: &[Field],
    close: usize,
    trailing: bool,
    added: &[String],
    indent: &str,
) {
    if added.is_empty() {
        return;
    }
    let mut prefix = "";
    if !trailing && let Some(last) = f.last() {
        if last.value.span.end == close {
            prefix = ",";
        } else {
            edits.push((last.value.span.end..last.value.span.end, ",".into()));
        }
    }
    edits.push((
        close..close,
        format!(
            "{prefix}\n{indent}{},\n",
            added.join(&format!(",\n{indent}"))
        ),
    ));
}
fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\{:03}", c as u32)),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}
fn binding_literal(index: usize, b: Option<&Binding>) -> String {
    match b {
        None => "nil".into(),
        Some(b) => {
            let mut s = format!(
                "{{ device = {}, input = {}",
                quote(&b.device),
                quote(&b.input)
            );
            if ACTIONS[index].axis {
                s.push_str(&format!(
                    ", invert = {}, mode = {}",
                    b.invert,
                    quote(if b.zero_to_one {
                        "zero_to_one"
                    } else {
                        "minus_one_to_one"
                    })
                ));
            }
            s.push_str(" }");
            s
        }
    }
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
    depth: usize,
}
impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.src.as_bytes().get(self.pos).copied()
    }
    fn err<T>(&self, msg: &str) -> Result<T, String> {
        Err(locate(self.src, self.pos, msg))
    }
    fn long_open(&self) -> Option<(usize, usize)> {
        let b = self.src.as_bytes();
        if b.get(self.pos) != Some(&b'[') {
            return None;
        }
        let mut n = self.pos + 1;
        while b.get(n) == Some(&b'=') {
            n += 1;
        }
        if b.get(n) == Some(&b'[') {
            Some((n - self.pos - 1, n + 1))
        } else {
            None
        }
    }
    fn long_string(&mut self) -> Result<String, String> {
        let (equals, start) = self.long_open().ok_or("Invalid long string")?;
        let end = format!("]{}]", "=".repeat(equals));
        let found = self.src[start..]
            .find(&end)
            .ok_or_else(|| locate(self.src, self.pos, "Unterminated long string/comment"))?;
        let mut content = &self.src[start..start + found];
        if content.starts_with("\r\n") {
            content = &content[2..]
        } else if content.starts_with(['\r', '\n']) {
            content = &content[1..]
        }
        self.pos = start + found + end.len();
        Ok(content.replace("\r\n", "\n").replace('\r', "\n"))
    }
    fn space(&mut self) -> Result<(), String> {
        loop {
            while self.peek().is_some_and(|b| b.is_ascii_whitespace()) {
                self.pos += 1
            }
            if self.src[self.pos..].starts_with("--") {
                self.pos += 2;
                if self.long_open().is_some() {
                    self.long_string()?;
                } else {
                    while self.peek().is_some_and(|b| b != b'\n' && b != b'\r') {
                        self.pos += 1
                    }
                }
            } else {
                return Ok(());
            }
        }
    }
    fn ident(&mut self) -> Result<String, String> {
        self.space()?;
        let start = self.pos;
        if !self
            .peek()
            .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        {
            return self.err("Expected a literal key or return");
        };
        while self
            .peek()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            self.pos += 1;
        }
        Ok(self.src[start..self.pos].into())
    }
    fn eat(&mut self, ch: u8) -> Result<(), String> {
        self.space()?;
        if self.peek() != Some(ch) {
            return self.err(&format!("Expected '{}'", ch as char));
        }
        self.pos += 1;
        Ok(())
    }
    fn quoted(&mut self) -> Result<String, String> {
        let delimiter = self.peek().unwrap();
        self.pos += 1;
        let mut bytes = Vec::new();
        loop {
            let b = self
                .peek()
                .ok_or_else(|| locate(self.src, self.pos, "Unterminated string"))?;
            self.pos += 1;
            if b == delimiter {
                break;
            }
            if matches!(b, b'\r' | b'\n') {
                return self.err("Unescaped newline in string");
            }
            if b != b'\\' {
                bytes.push(b);
                continue;
            }
            let e = self.peek().ok_or("Unterminated escape")?;
            self.pos += 1;
            match e {
                b'a' => bytes.push(7),
                b'b' => bytes.push(8),
                b'f' => bytes.push(12),
                b'n' => bytes.push(10),
                b'r' => bytes.push(13),
                b't' => bytes.push(9),
                b'v' => bytes.push(11),
                b'\\' | b'\'' | b'"' => bytes.push(e),
                b'\n' => bytes.push(10),
                b'\r' => {
                    if self.peek() == Some(b'\n') {
                        self.pos += 1;
                    }
                    bytes.push(10)
                }
                b'z' => {
                    while self.peek().is_some_and(|c| c.is_ascii_whitespace()) {
                        self.pos += 1
                    }
                }
                b'x' => {
                    let end = self.pos + 2;
                    let h = self.src.get(self.pos..end).ok_or("Invalid hex escape")?;
                    bytes.push(u8::from_str_radix(h, 16).map_err(|_| "Invalid hex escape")?);
                    self.pos = end
                }
                b'u' => {
                    self.eat(b'{')?;
                    let start = self.pos;
                    while self.peek().is_some_and(|b| b.is_ascii_hexdigit()) {
                        self.pos += 1;
                    }
                    let code = u32::from_str_radix(&self.src[start..self.pos], 16)
                        .map_err(|_| "Invalid Unicode escape")?;
                    self.eat(b'}')?;
                    let c = char::from_u32(code).ok_or("Invalid Unicode scalar")?;
                    let mut buf = [0; 4];
                    bytes.extend(c.encode_utf8(&mut buf).as_bytes());
                }
                b'0'..=b'9' => {
                    let start = self.pos - 1;
                    while self.pos - start < 3 && self.peek().is_some_and(|b| b.is_ascii_digit()) {
                        self.pos += 1;
                    }
                    let n = self.src[start..self.pos]
                        .parse::<u8>()
                        .map_err(|_| "Decimal escape exceeds 255")?;
                    bytes.push(n)
                }
                _ => return self.err("Unsupported string escape"),
            }
        }
        String::from_utf8(bytes).map_err(|_| "String is not valid UTF-8".into())
    }
    fn value(&mut self) -> Result<Value, String> {
        self.space()?;
        let start = self.pos;
        let kind = match self.peek() {
            Some(b'{') => {
                self.depth += 1;
                if self.depth > 16 {
                    return self.err("Table nesting exceeds 16");
                };
                self.pos += 1;
                let mut f = Vec::new();
                let mut keys = BTreeSet::new();
                let mut trailing = false;
                loop {
                    self.space()?;
                    if self.peek() == Some(b'}') {
                        break;
                    }
                    let key = if self.peek() == Some(b'[') {
                        self.pos += 1;
                        self.space()?;
                        let k = if matches!(self.peek(), Some(b'\'' | b'"')) {
                            self.quoted()?
                        } else if self.long_open().is_some() {
                            self.long_string()?
                        } else {
                            return self.err("Only literal string keys are supported");
                        };
                        self.eat(b']')?;
                        k
                    } else {
                        self.ident()?
                    };
                    if !keys.insert(key.clone()) {
                        return self.err(&format!("Duplicate key: {key}"));
                    }
                    self.eat(b'=')?;
                    f.push(Field {
                        key,
                        value: self.value()?,
                    });
                    self.space()?;
                    trailing = matches!(self.peek(), Some(b',' | b';'));
                    if trailing {
                        self.pos += 1
                    } else if self.peek() != Some(b'}') {
                        return self.err("Expected ',' or '}'");
                    }
                }
                let close = self.pos;
                self.pos += 1;
                self.depth -= 1;
                Kind::Table(f, close, trailing)
            }
            Some(b'\'' | b'"') => Kind::String(self.quoted()?),
            Some(b'[') if self.long_open().is_some() => Kind::String(self.long_string()?),
            Some(b'-' | b'0'..=b'9') => {
                if self.peek() == Some(b'-') {
                    self.pos += 1;
                }
                while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                    self.pos += 1;
                }
                if self.peek() == Some(b'.') {
                    self.pos += 1;
                    while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                        self.pos += 1;
                    }
                }
                if matches!(self.peek(), Some(b'e' | b'E')) {
                    self.pos += 1;
                    if matches!(self.peek(), Some(b'+' | b'-')) {
                        self.pos += 1;
                    }
                    while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                        self.pos += 1;
                    }
                }
                let n = self.src[start..self.pos]
                    .parse::<f64>()
                    .map_err(|_| locate(self.src, start, "Invalid number"))?;
                if !n.is_finite() {
                    return self.err("Non-finite number");
                };
                Kind::Number(n)
            }
            Some(_) => match self.ident()?.as_str() {
                "nil" => Kind::Nil,
                "true" => Kind::Bool(true),
                "false" => Kind::Bool(false),
                _ => {
                    return self.err("Only table literals are supported; Lua code is not executed");
                }
            },
            None => return self.err("Unexpected end of configuration"),
        };
        Ok(Value {
            kind,
            span: start..self.pos,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const GUID: &str = "{11111111-2222-3333-4444-555555555555}";
    fn sample() -> String {
        format!(
            "-- header\r\nreturn {{version=1, trace_throttle=true, bindings={{\r\nRoll={{device='{GUID}', input='X', -- preserve this\r\ninvert=false,mode='signed'}}, Gun=nil, -- trailing\r\n}}}}\r\n-- BEGIN DETECTED DEVICES (generated; edit bindings above)\r\n-- Inventory stays here\r\n"
        )
    }
    #[test]
    fn exact_unedited_roundtrip() {
        let text = sample();
        let d = Document::parse(text.clone()).unwrap();
        assert_eq!(d.render(&d.bindings).unwrap(), text);
        let t = Document::parse(include_str!("../../mod/bindings.lua").into()).unwrap();
        assert_eq!(t.bindings.len(), 52);
        assert!(t.bindings.iter().all(Option::is_none));
    }
    #[test]
    fn edits_preserve_comments_aliases_and_suffix() {
        let d = Document::parse(sample()).unwrap();
        let mut b = d.bindings.clone();
        b[0].as_mut().unwrap().invert = true;
        b[6] = Some(Binding {
            device: GUID.into(),
            input: "Button128".into(),
            invert: false,
            zero_to_one: false,
        });
        let output = d.render(&b).unwrap();
        assert!(output.contains("-- preserve this\r\n"));
        assert!(output.contains("mode='signed'"));
        assert!(output.ends_with("-- Inventory stays here\r\n"));
        assert_eq!(Document::parse(output).unwrap().bindings, b);
    }
    #[test]
    fn inserts_sparse_keys_and_options_without_losing_comments() {
        let d = Document::parse(format!(
            "return {{version=1,bindings={{ Roll={{device='{GUID}',input='X'}} -- last\n}}}}"
        ))
        .unwrap();
        let mut b = d.bindings.clone();
        b[0].as_mut().unwrap().invert = true;
        b[3] = Some(Binding {
            device: GUID.into(),
            input: "Slider2".into(),
            invert: true,
            zero_to_one: true,
        });
        let output = d.render(&b).unwrap();
        assert!(output.contains("-- last"));
        assert_eq!(Document::parse(output).unwrap().bindings, b);
    }
    #[test]
    fn rejects_executable_malformed_unknown_and_duplicate_configs() {
        for text in [
            "return (function()return {}end)()",
            "return {version=1,bindings={}} os.execute('bad')",
            "return {version=2,bindings={}}",
            "return {version=1,bindings={},other=true}",
            "return {version=1,version=1,bindings={}}",
            "return {version=1,bindings={NoSuchAction=nil}}",
            "return {version=1,bindings={Roll=make()}}",
            "return {version=1,bindings={}",
        ] {
            assert!(Document::parse(text.into()).is_err(), "{text}");
        }
    }
    #[test]
    fn understands_literal_keys_and_lua_string_forms() {
        let text = format!(
            "--[=[long comment]=]\nreturn {{['version']=1.0,bindings={{[\"Roll\"]={{device=[=[{GUID}]=],input='\\x58',mode='signed'}}}}}}"
        );
        let d = Document::parse(text).unwrap();
        assert_eq!(d.bindings[0].as_ref().unwrap().input, "X");
        assert!(
            Document::parse(format!(
                "return {{version=1,bindings={{Gun={{device='{GUID}',input='Button01'}}}}}}"
            ))
            .is_err()
        );
    }
    #[test]
    fn rejects_axis_options_on_buttons_and_invalid_tokens() {
        for extra in [
            "input='X'",
            "input='Button129'",
            "input='POV2_Up'",
            "input='Button1',mode='signed'",
            "input='Button1',invert=true",
        ] {
            assert!(
                Document::parse(format!(
                    "return {{version=1,bindings={{Gun={{device='{GUID}',{extra}}}}}}}"
                ))
                .is_err()
            );
        }
    }
    #[test]
    fn bom_adjacent_comments_and_semicolons_roundtrip() {
        let text = "\u{feff}return {version=1-- adjacent comment\n;bindings={};trace_throttle=1e-2}; -- tail";
        let d = Document::parse(text.into()).unwrap();
        assert_eq!(d.render(&d.bindings).unwrap(), text);
    }
}
