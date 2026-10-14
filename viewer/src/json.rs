//! A JSON reader, enough for the replay format: objects, arrays, strings with
//! escapes, numbers, booleans and null. Errors carry the byte offset.

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Value>),
    /// **An array of numbers only**, held as numbers (T-180): a replay's
    /// rows are almost all of this kind, and a [`Value`] per number made the
    /// parsed document ten times its text. Read with [`Value::num_at`] or
    /// [`Value::nums`], which take either form.
    Nums(Vec<f64>),
    Obj(Vec<(String, Value)>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(v) => v.iter().find(|(k, _)| k == key).map(|(_, x)| x),
            _ => None,
        }
    }
    pub fn as_arr(&self) -> Option<&[Value]> {
        match self {
            Value::Arr(v) => Some(v),
            _ => None,
        }
    }
    /// Element `i` of an array, as a number: `None` past the end or where it
    /// is not a number.
    pub fn num_at(&self, i: usize) -> Option<f64> {
        match self {
            Value::Nums(v) => v.get(i).copied(),
            Value::Arr(v) => v.get(i).and_then(Value::as_f64),
            _ => None,
        }
    }
    /// The numbers of an array, skipping anything else; empty for a non-array.
    pub fn nums(&self) -> Vec<f64> {
        match self {
            Value::Nums(v) => v.clone(),
            Value::Arr(v) => v.iter().filter_map(Value::as_f64).collect(),
            _ => Vec::new(),
        }
    }
    /// Whether this is an array, of either form.
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Arr(_) | Value::Nums(_))
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Num(x) => Some(*x),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

pub fn parse(text: &str) -> Result<Value, String> {
    let mut p = Parser { b: text.as_bytes(), i: 0 };
    let v = p.value()?;
    p.ws();
    if p.i != p.b.len() {
        return Err(format!("trailing bytes at {}", p.i));
    }
    Ok(v)
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.b.len() && self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn peek(&self) -> Result<u8, String> {
        self.b.get(self.i).copied().ok_or_else(|| format!("unexpected end at {}", self.i))
    }

    fn expect(&mut self, c: u8) -> Result<(), String> {
        if self.peek()? != c {
            return Err(format!("expected '{}' at {}", c as char, self.i));
        }
        self.i += 1;
        Ok(())
    }

    fn literal(&mut self, word: &str, v: Value) -> Result<Value, String> {
        if self.b[self.i..].starts_with(word.as_bytes()) {
            self.i += word.len();
            Ok(v)
        } else {
            Err(format!("bad literal at {}", self.i))
        }
    }

    fn value(&mut self) -> Result<Value, String> {
        self.ws();
        match self.peek()? {
            b'{' => self.object(),
            b'[' => self.array(),
            b'"' => self.string().map(Value::Str),
            b't' => self.literal("true", Value::Bool(true)),
            b'f' => self.literal("false", Value::Bool(false)),
            b'n' => self.literal("null", Value::Null),
            _ => self.number(),
        }
    }

    fn object(&mut self) -> Result<Value, String> {
        self.expect(b'{')?;
        let mut out = Vec::new();
        self.ws();
        if self.peek()? == b'}' {
            self.i += 1;
            return Ok(Value::Obj(out));
        }
        loop {
            self.ws();
            let k = self.string()?;
            self.ws();
            self.expect(b':')?;
            out.push((k, self.value()?));
            self.ws();
            match self.peek()? {
                b',' => self.i += 1,
                b'}' => {
                    self.i += 1;
                    return Ok(Value::Obj(out));
                }
                _ => return Err(format!("expected ',' or '}}' at {}", self.i)),
            }
        }
    }

    /// An array, held as numbers while every element so far is one: a
    /// `Vec<f64>` costs 8 bytes a number where a `Value` costs 32, and a
    /// replay is mostly rows of numbers.
    fn array(&mut self) -> Result<Value, String> {
        self.expect(b'[')?;
        let mut nums: Vec<f64> = Vec::new();
        let mut vals: Option<Vec<Value>> = None;
        self.ws();
        if self.peek()? == b']' {
            self.i += 1;
            return Ok(Value::Arr(Vec::new()));
        }
        loop {
            match (self.value()?, &mut vals) {
                (Value::Num(x), None) => nums.push(x),
                (v, Some(vs)) => vs.push(v),
                (v, None) => {
                    let mut vs: Vec<Value> = nums.drain(..).map(Value::Num).collect();
                    vs.push(v);
                    vals = Some(vs);
                }
            }
            self.ws();
            match self.peek()? {
                b',' => self.i += 1,
                b']' => {
                    self.i += 1;
                    return Ok(match vals {
                        Some(mut vs) => {
                            vs.shrink_to_fit();
                            Value::Arr(vs)
                        }
                        None => {
                            nums.shrink_to_fit();
                            Value::Nums(nums)
                        }
                    });
                }
                _ => return Err(format!("expected ',' or ']' at {}", self.i)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let c = self.peek()?;
            self.i += 1;
            match c {
                b'"' => return String::from_utf8(out).map_err(|_| format!("bad UTF-8 before {}", self.i)),
                b'\\' => {
                    let e = self.peek()?;
                    self.i += 1;
                    match e {
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'b' => out.push(8),
                        b'f' => out.push(12),
                        b'u' => {
                            let hex = std::str::from_utf8(self.b.get(self.i..self.i + 4).ok_or("short \\u escape")?)
                                .map_err(|_| "bad \\u escape")?;
                            let n = u32::from_str_radix(hex, 16).map_err(|_| format!("bad \\u at {}", self.i))?;
                            self.i += 4;
                            let ch = char::from_u32(n).unwrap_or('\u{fffd}');
                            let mut buf = [0u8; 4];
                            out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                        }
                        other => out.push(other),
                    }
                }
                _ => out.push(c),
            }
        }
    }

    fn number(&mut self) -> Result<Value, String> {
        let start = self.i;
        while self.i < self.b.len() && matches!(self.b[self.i], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
            self.i += 1;
        }
        std::str::from_utf8(&self.b[start..self.i])
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .map(Value::Num)
            .ok_or_else(|| format!("bad number at {start}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_every_kind_of_value() {
        let v =
            parse(r#" {"a": [1, -2.5, 3e2], "b": "x\"y\\z\né", "c": true, "d": false, "e": null, "f": {}} "#).unwrap();
        // An array of numbers only is held as numbers; a mixed one as values.
        assert_eq!(v.get("a"), Some(&Value::Nums(vec![1.0, -2.5, 300.0])));
        assert_eq!(v.get("a").unwrap().nums(), [1.0, -2.5, 300.0]);
        assert_eq!(v.get("a").unwrap().num_at(1), Some(-2.5));
        let mixed = parse("[1, null, 3]").unwrap();
        assert!(mixed.as_arr().is_some());
        assert_eq!((mixed.num_at(0), mixed.num_at(1), mixed.num_at(2)), (Some(1.0), None, Some(3.0)));
        assert_eq!(mixed.nums(), [1.0, 3.0]);
        assert_eq!(v.get("b").unwrap().as_str(), Some("x\"y\\z\né"));
        assert_eq!(v.get("c").unwrap().as_bool(), Some(true));
        assert_eq!(v.get("d").unwrap().as_bool(), Some(false));
        assert_eq!(v.get("e"), Some(&Value::Null));
        assert_eq!(v.get("f"), Some(&Value::Obj(vec![])));
    }

    #[test]
    fn keeps_multibyte_text_whole() {
        assert_eq!(parse("\"Δ — ly\"").unwrap().as_str(), Some("Δ — ly"));
    }

    #[test]
    fn says_where_a_malformed_document_breaks() {
        for bad in ["{\"a\" 1}", "[1, 2", "\"open", "{\"a\": tru}", "1 2"] {
            assert!(parse(bad).is_err(), "{bad}");
        }
        assert!(parse("[1, 2").unwrap_err().contains("end"));
    }
}
