#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! Zero-copy logfmt parser. See [`LogfmtParser`] for the streaming iterator API,
//! [`parse_to_map`] and [`parse_to_pairs`] for the convenience owned APIs, and
//! [`LogfmtLinesReader`] for multi-line sources.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::io::{self, BufRead};

/// An error produced during logfmt parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogfmtError {
    /// Byte offset in the input where the error occurred.
    pub position: usize,
    /// Human-readable description of what went wrong.
    pub message: &'static str,
}

impl fmt::Display for LogfmtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "logfmt parse error at position {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for LogfmtError {}

fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\r' | b'\n')
}

fn is_token_break(byte: u8) -> bool {
    is_whitespace(byte) || matches!(byte, b'=' | b'"')
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// Zero-copy streaming logfmt parser.
///
/// Yields `(Cow<str>, Cow<str>)` key-value pairs borrowed directly from the
/// input; a token is only copied when it contains escape sequences that have to
/// be decoded. Keys without a `=` are yielded with an empty-string value.
///
/// Values (and keys) may be quoted, in which case `\"`, `\\`, `\n`, `\r`, `\t`
/// and `\xNN` escapes are decoded — exactly like [`parse_to_map`] and
/// [`parse_to_pairs`], which are implemented on top of this iterator.
///
/// Whitespace (space, tab, carriage return, line feed) separates tokens, so a
/// trailing newline never becomes part of the final value.
///
/// # Example
///
/// ```
/// use tpt_logfmt_parse::LogfmtParser;
///
/// let input = r#"level=info msg="hello world" latency=42ms"#;
/// let pairs: Vec<_> = LogfmtParser::new(input).collect::<Result<_, _>>().unwrap();
/// assert_eq!(pairs[0].0, "level");
/// assert_eq!(pairs[0].1, "info");
/// assert_eq!(pairs[1].1, "hello world");
/// assert_eq!(pairs[2].1, "42ms");
/// ```
pub struct LogfmtParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> LogfmtParser<'a> {
    /// Create a new parser for the given logfmt line.
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(is_whitespace) {
            self.pos += 1;
        }
    }

    /// Parse an unquoted token (key or bare value): everything up to `=`, `"`,
    /// or whitespace.
    fn parse_bare(&mut self) -> &'a str {
        let start = self.pos;
        while self.peek().is_some_and(|b| !is_token_break(b)) {
            self.pos += 1;
        }
        &self.input[start..self.pos]
    }

    /// Parse a quoted token, returning its decoded content.
    ///
    /// The result borrows straight from the input when the token contains no
    /// escape sequences; otherwise the decoded content is allocated. This is
    /// the single decoding path shared by every public API of this crate, so
    /// the iterator and the owned helpers can never disagree.
    fn parse_quoted(&mut self) -> Result<Cow<'a, str>, LogfmtError> {
        debug_assert_eq!(self.peek(), Some(b'"'));
        self.pos += 1; // consume opening quote
        let mut chunk_start = self.pos;
        let mut decoded: Option<String> = None;
        loop {
            let Some(byte) = self.peek() else {
                return Err(LogfmtError {
                    position: self.pos,
                    message: "unterminated quoted string",
                });
            };
            match byte {
                b'"' => {
                    let tail = &self.input[chunk_start..self.pos];
                    self.pos += 1; // consume closing quote
                    return Ok(match decoded {
                        Some(mut buf) => {
                            buf.push_str(tail);
                            Cow::Owned(buf)
                        }
                        None => Cow::Borrowed(tail),
                    });
                }
                b'\\' => {
                    let mut buf = decoded.take().unwrap_or_default();
                    buf.push_str(&self.input[chunk_start..self.pos]);
                    self.pos += 1;
                    let Some(escaped) = self.input[self.pos..].chars().next() else {
                        return Err(LogfmtError {
                            position: self.pos,
                            message: "unterminated escape sequence",
                        });
                    };
                    self.pos += escaped.len_utf8();
                    match escaped {
                        '"' => buf.push('"'),
                        '\\' => buf.push('\\'),
                        'n' => buf.push('\n'),
                        'r' => buf.push('\r'),
                        't' => buf.push('\t'),
                        'x' => self.decode_hex_escape(&mut buf),
                        other => {
                            buf.push('\\');
                            buf.push(other);
                        }
                    }
                    chunk_start = self.pos;
                    decoded = Some(buf);
                }
                _ => self.pos += 1,
            }
        }
    }

    fn decode_hex_escape(&mut self, buf: &mut String) {
        let bytes = self.input.as_bytes();
        let high = bytes.get(self.pos).copied().and_then(hex_digit);
        let low = bytes.get(self.pos + 1).copied().and_then(hex_digit);
        if let (Some(high), Some(low)) = (high, low) {
            buf.push(char::from(high * 16 + low));
            self.pos += 2;
        } else {
            buf.push('\\');
            buf.push('x');
        }
    }

    /// Consume the token the parser choked on so iteration always progresses.
    fn consume_invalid_token(&mut self) {
        if self.peek() == Some(b'=') {
            self.pos += 1;
        }
        if self.peek() == Some(b'"') {
            let _ = self.parse_quoted();
        } else {
            let _ = self.parse_bare();
        }
    }
}

impl<'a> Iterator for LogfmtParser<'a> {
    type Item = Result<(Cow<'a, str>, Cow<'a, str>), LogfmtError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        let first = self.peek()?;

        let key = if first == b'"' {
            match self.parse_quoted() {
                Ok(key) => key,
                Err(err) => return Some(Err(err)),
            }
        } else {
            let key = self.parse_bare();
            if key.is_empty() {
                let position = self.pos;
                self.consume_invalid_token();
                return Some(Err(LogfmtError {
                    position,
                    message: "expected key",
                }));
            }
            Cow::Borrowed(key)
        };

        // No `=` — bare key with empty value
        if self.peek() != Some(b'=') {
            return Some(Ok((key, Cow::Borrowed(""))));
        }
        self.pos += 1; // consume `=`

        // Value: quoted or bare
        if self.peek() == Some(b'"') {
            Some(self.parse_quoted().map(|value| (key, value)))
        } else {
            Some(Ok((key, Cow::Borrowed(self.parse_bare()))))
        }
    }
}

/// Parse a logfmt string into an owned `HashMap<String, String>`.
///
/// Escape sequences inside quoted tokens (`\"`, `\\`, `\n`, `\r`, `\t`,
/// `\xNN`) are resolved. Duplicate keys collapse — the last occurrence wins;
/// use [`parse_to_pairs`] to keep order and duplicates.
///
/// # Example
///
/// ```
/// use tpt_logfmt_parse::parse_to_map;
///
/// let map = parse_to_map(r#"level=error msg="disk full" retries=3"#).unwrap();
/// assert_eq!(map["level"], "error");
/// assert_eq!(map["msg"], "disk full");
/// assert_eq!(map["retries"], "3");
/// ```
pub fn parse_to_map(input: &str) -> Result<HashMap<String, String>, LogfmtError> {
    let mut map = HashMap::new();
    for pair in LogfmtParser::new(input) {
        let (key, value) = pair?;
        map.insert(key.into_owned(), value.into_owned());
    }
    Ok(map)
}

/// Parse a logfmt string into an owned `Vec<(String, String)>`.
///
/// Order-preserving, duplicate-preserving sibling of [`parse_to_map`]: pairs
/// come back in input order and repeated keys are all retained. Escape
/// sequences are resolved exactly as [`LogfmtParser`] resolves them.
///
/// # Example
///
/// ```
/// use tpt_logfmt_parse::parse_to_pairs;
///
/// let pairs = parse_to_pairs("tag=a tag=b level=info").unwrap();
/// assert_eq!(
///     pairs,
///     vec![
///         ("tag".to_string(), "a".to_string()),
///         ("tag".to_string(), "b".to_string()),
///         ("level".to_string(), "info".to_string()),
///     ]
/// );
/// ```
pub fn parse_to_pairs(input: &str) -> Result<Vec<(String, String)>, LogfmtError> {
    let mut pairs = Vec::new();
    for pair in LogfmtParser::new(input) {
        let (key, value) = pair?;
        pairs.push((key.into_owned(), value.into_owned()));
    }
    Ok(pairs)
}

/// The kind of error that occurred while reading a multi-line logfmt stream.
#[derive(Debug)]
pub enum LogfmtLinesErrorKind {
    /// An I/O error from the underlying reader.
    Io(io::Error),
    /// A logfmt parse error on a specific line.
    Parse(LogfmtError),
}

impl fmt::Display for LogfmtLinesErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Parse(e) => write!(f, "{}", e),
        }
    }
}

/// An error produced while reading or parsing a multi-line logfmt stream.
///
/// Includes the 1-based line number where the error occurred.
#[derive(Debug)]
pub struct LogfmtLinesError {
    /// The 1-based line number where the error occurred.
    pub line: u64,
    /// The underlying error kind.
    pub kind: LogfmtLinesErrorKind,
}

impl fmt::Display for LogfmtLinesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "logfmt error on line {}: {}", self.line, self.kind)
    }
}

impl std::error::Error for LogfmtLinesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            LogfmtLinesErrorKind::Io(e) => Some(e),
            LogfmtLinesErrorKind::Parse(e) => Some(e),
        }
    }
}

/// A streaming reader for multi-line logfmt sources (Heroku router logs, etc.).
///
/// Wraps any [`BufRead`] and yields one `Vec<(String, String)>` per non-empty
/// line, in input order and keeping duplicate keys. Blank lines are silently
/// skipped. Errors carry the 1-based line number.
///
/// # Example
///
/// ```
/// use tpt_logfmt_parse::LogfmtLinesReader;
/// use std::io::BufReader;
///
/// let data = b"at=info status=200\nat=warn status=503\n";
/// let mut reader = LogfmtLinesReader::new(BufReader::new(data.as_slice()));
/// let first = reader.next().unwrap().unwrap();
/// assert_eq!(first[0], ("at".to_string(), "info".to_string()));
/// assert_eq!(first[1], ("status".to_string(), "200".to_string()));
/// ```
pub struct LogfmtLinesReader<R: BufRead> {
    reader: R,
    buf: String,
    line: u64,
}

impl<R: BufRead> LogfmtLinesReader<R> {
    /// Create a new `LogfmtLinesReader` wrapping the given buffered reader.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: String::new(),
            line: 0,
        }
    }

    /// The 1-based line number most recently read (or 0 before any reads).
    pub fn line_number(&self) -> u64 {
        self.line
    }
}

impl<R: BufRead> Iterator for LogfmtLinesReader<R> {
    type Item = Result<Vec<(String, String)>, LogfmtLinesError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();
            match self.reader.read_line(&mut self.buf) {
                Err(e) => {
                    self.line += 1;
                    return Some(Err(LogfmtLinesError {
                        line: self.line,
                        kind: LogfmtLinesErrorKind::Io(e),
                    }));
                }
                Ok(0) => return None,
                Ok(_) => {
                    self.line += 1;
                    let line = self.line;
                    let trimmed = self.buf.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    return Some(parse_to_pairs(trimmed).map_err(|e| LogfmtLinesError {
                        line,
                        kind: LogfmtLinesErrorKind::Parse(e),
                    }));
                }
            }
        }
    }
}

/// Create a [`LogfmtLinesReader`] from any [`BufRead`].
///
/// Convenience wrapper around [`LogfmtLinesReader::new`].
///
/// # Example
///
/// ```
/// use tpt_logfmt_parse::parse_logfmt_lines;
/// use std::io::BufReader;
///
/// let data = b"a=1\n\nb=2\n";
/// let lines: Vec<_> = parse_logfmt_lines(BufReader::new(data.as_slice()))
///     .collect::<Result<_, _>>()
///     .unwrap();
/// assert_eq!(lines.len(), 2);
/// ```
pub fn parse_logfmt_lines<R: BufRead>(reader: R) -> LogfmtLinesReader<R> {
    LogfmtLinesReader::new(reader)
}

/// Serialize a single `key=value` field into logfmt form.
///
/// The key and value are quoted when they are empty or contain a space, quote,
/// backslash, `=`, or any control character; inside quotes `"`, `\`, and
/// control characters are escaped (`\"`, `\\`, `\n`, `\r`, `\t`, `\xNN`). The
/// output always parses back to the original key and value.
///
/// # Example
///
/// ```
/// use tpt_logfmt_parse::format_pair;
/// assert_eq!(format_pair("level", "info"), "level=info");
/// assert_eq!(format_pair("msg", "hello world"), r#"msg="hello world""#);
/// assert_eq!(format_pair("msg", "a\nb"), r#"msg="a\nb""#);
/// ```
pub fn format_pair(key: &str, value: &str) -> String {
    let mut out = String::with_capacity(key.len() + value.len() + 3);
    push_token(&mut out, key);
    out.push('=');
    push_token(&mut out, value);
    out
}

/// Serialize an iterator of `(key, value)` pairs into a single logfmt line.
///
/// Every token is quoted and escaped as needed, so the result always round-trips
/// through [`parse_to_pairs`] (and through [`parse_to_map`] for unique keys).
///
/// # Example
///
/// ```
/// use tpt_logfmt_parse::{parse_to_pairs, write_logfmt};
///
/// let line = write_logfmt(vec![("level", "info"), ("msg", "hello world")]);
/// assert_eq!(line, r#"level=info msg="hello world""#);
///
/// let pairs = vec![("odd key".to_string(), "a\t\"b\"".to_string())];
/// assert_eq!(parse_to_pairs(&write_logfmt(pairs.clone())).unwrap(), pairs);
/// ```
pub fn write_logfmt<I, K, V>(pairs: I) -> String
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut out = String::new();
    for (i, (k, v)) in pairs.into_iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(&format_pair(k.as_ref(), v.as_ref()));
    }
    out
}

fn needs_quoting(s: &str) -> bool {
    s.is_empty()
        || s.chars()
            .any(|c| matches!(c, ' ' | '"' | '\\' | '=') || c.is_control())
}

fn push_token(out: &mut String, s: &str) {
    if !needs_quoting(s) {
        out.push_str(s);
        return;
    }
    out.reserve(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other if other.is_control() => push_hex_escape(out, other),
            other => out.push(other),
        }
    }
    out.push('"');
}

fn push_hex_escape(out: &mut String, c: char) {
    const HEX: [char; 16] = [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
    ];
    let code = c as u32;
    out.push('\\');
    out.push('x');
    out.push(HEX[((code >> 4) & 0xf) as usize]);
    out.push(HEX[(code & 0xf) as usize]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn owned(input: &str) -> Vec<(String, String)> {
        parse_to_pairs(input).unwrap()
    }

    fn pair(key: &str, value: &str) -> (String, String) {
        (key.to_string(), value.to_string())
    }

    #[test]
    fn basic_pairs() {
        assert_eq!(owned("a=1 b=2"), vec![pair("a", "1"), pair("b", "2")]);
    }

    #[test]
    fn quoted_value() {
        assert_eq!(
            owned(r#"msg="hello world""#),
            vec![pair("msg", "hello world")]
        );
    }

    #[test]
    fn bare_key_no_value() {
        assert_eq!(owned("flag"), vec![pair("flag", "")]);
    }

    #[test]
    fn mixed() {
        assert_eq!(
            owned(r#"level=info flag msg="ok" count=3"#),
            vec![
                pair("level", "info"),
                pair("flag", ""),
                pair("msg", "ok"),
                pair("count", "3"),
            ]
        );
    }

    #[test]
    fn empty_input() {
        assert!(owned("").is_empty());
    }

    #[test]
    fn unterminated_quote_error() {
        let result: Result<Vec<_>, _> = LogfmtParser::new(r#"msg="unclosed"#).collect();
        assert!(result.is_err());
    }

    #[test]
    fn parse_to_map_escapes() {
        let map = parse_to_map(r#"msg="say \"hi\"" path="a\\b""#).unwrap();
        assert_eq!(map["msg"], r#"say "hi""#);
        assert_eq!(map["path"], r"a\b");
    }

    #[test]
    fn parse_to_map_bare_key() {
        let map = parse_to_map("enabled").unwrap();
        assert_eq!(map["enabled"], "");
    }

    #[test]
    fn parse_to_map_roundtrip() {
        let input = r#"level=error msg="disk full" retries=3"#;
        let map = parse_to_map(input).unwrap();
        assert_eq!(map["level"], "error");
        assert_eq!(map["msg"], "disk full");
        assert_eq!(map["retries"], "3");
    }

    #[test]
    fn parse_to_map_multibyte_utf8() {
        let map = parse_to_map("msg=\"café ☕ résumé\"").unwrap();
        assert_eq!(map["msg"], "café ☕ résumé");
    }

    #[test]
    fn logfmt_parser_multibyte_utf8() {
        assert_eq!(owned("msg=\"café ☕\""), vec![pair("msg", "café ☕")]);
    }

    #[test]
    fn iterator_advances_past_expected_key_error() {
        let mut parser = LogfmtParser::new("=1 b=2");
        let err = parser.next().unwrap().unwrap_err();
        assert_eq!(err.message, "expected key");
        assert_eq!(err.position, 0);
        let (key, value) = parser.next().unwrap().unwrap();
        assert_eq!(key, "b");
        assert_eq!(value, "2");
        assert!(parser.next().is_none());
    }

    #[test]
    fn iterator_terminates_on_repeated_errors() {
        let items: Vec<_> = LogfmtParser::new("= = =").take(16).collect();
        assert_eq!(items.len(), 3);
        assert!(items.iter().all(|item| item.is_err()));
    }

    #[test]
    fn iterator_advances_past_quoted_error_token() {
        let mut parser = LogfmtParser::new(r#"="v" next=1"#);
        assert!(parser.next().unwrap().is_err());
        let (key, value) = parser.next().unwrap().unwrap();
        assert_eq!(key, "next");
        assert_eq!(value, "1");
    }

    #[test]
    fn crlf_is_whitespace() {
        let map = parse_to_map("a=1\r\n").unwrap();
        assert_eq!(map.len(), 1);
        assert_eq!(map["a"], "1");
    }

    #[test]
    fn tabs_separate_tokens() {
        assert_eq!(owned("a=1\tb=2"), vec![pair("a", "1"), pair("b", "2")]);
    }

    #[test]
    fn multi_line_input_with_tabs() {
        let input = "a=1\tb=2\r\nc=3\n\td=4\n";
        assert_eq!(
            owned(input),
            vec![
                pair("a", "1"),
                pair("b", "2"),
                pair("c", "3"),
                pair("d", "4")
            ]
        );
    }

    #[test]
    fn quoted_value_keeps_escaped_whitespace() {
        let map = parse_to_map(r#"msg="a\tb\r\nc" k=v"#).unwrap();
        assert_eq!(map["msg"], "a\tb\r\nc");
        assert_eq!(map["k"], "v");
    }

    #[test]
    fn parser_and_map_agree_on_escapes() {
        let input = r#"msg="say \"hi\"" path="a\\b" hex="x\x07y""#;
        let map = parse_to_map(input).unwrap();
        for (key, value) in owned(input) {
            assert_eq!(map[&key], value);
        }
        assert_eq!(map["msg"], r#"say "hi""#);
        assert_eq!(map["path"], r"a\b");
        assert_eq!(map["hex"], "x\u{7}y");
    }

    #[test]
    fn parser_borrows_when_no_escapes() {
        let input = r#"msg="hello world""#;
        let (_, value) = LogfmtParser::new(input).next().unwrap().unwrap();
        assert!(matches!(value, Cow::Borrowed(_)));
    }

    #[test]
    fn quoted_key_is_supported() {
        let pairs = owned(r#""a b"="x y" "k\"q"=1"#);
        assert_eq!(pairs, vec![pair("a b", "x y"), pair(r#"k"q"#, "1")]);
    }

    #[test]
    fn parse_to_pairs_preserves_order_and_duplicates() {
        let pairs = owned("a=1 a=2 b=3");
        assert_eq!(pairs, vec![pair("a", "1"), pair("a", "2"), pair("b", "3")]);
        let map = parse_to_map("a=1 a=2 b=3").unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map["a"], "2");
    }

    #[test]
    fn parse_to_pairs_propagates_errors() {
        assert!(parse_to_pairs(r#"msg="unclosed"#).is_err());
    }

    #[test]
    fn write_logfmt_basic() {
        let line = write_logfmt(vec![("level", "info"), ("msg", "hello world")]);
        assert_eq!(line, r#"level=info msg="hello world""#);
    }

    #[test]
    fn write_logfmt_quotes_special_chars() {
        assert_eq!(
            write_logfmt(vec![("msg", r#"say "hi""#)]),
            r#"msg="say \"hi\"""#
        );
        assert_eq!(write_logfmt(vec![("a", "b=c")]), r#"a="b=c""#);
        assert_eq!(write_logfmt(vec![("empty", "")]), r#"empty="""#);
    }

    #[test]
    fn write_logfmt_quotes_keys_that_need_it() {
        assert_eq!(write_logfmt(vec![("a b", "x")]), r#""a b"=x"#);
        assert_eq!(
            parse_to_map(&write_logfmt(vec![("a b", "x")])).unwrap()["a b"],
            "x"
        );
    }

    #[test]
    fn write_logfmt_escapes_control_chars() {
        assert_eq!(write_logfmt(vec![("msg", "a\nb")]), r#"msg="a\nb""#);
        assert_eq!(write_logfmt(vec![("msg", "a\tb")]), r#"msg="a\tb""#);
        assert_eq!(write_logfmt(vec![("msg", "a\rb")]), r#"msg="a\rb""#);
        assert_eq!(write_logfmt(vec![("msg", "a\u{7}b")]), r#"msg="a\x07b""#);
        assert!(!write_logfmt(vec![("msg", "a\nb")]).contains('\n'));
    }

    #[test]
    fn write_logfmt_round_trips_with_parser() {
        let pairs = vec![
            pair("level", "info"),
            pair("msg", "hello world"),
            pair("a b", "x y"),
            pair("quote", r#"say "hi""#),
            pair("back\\slash", r"a\b"),
            pair("ctrl", "line1\nline2\ttab\rcr\u{1}\u{7f}"),
            pair("empty", ""),
            pair("eq", "k=v"),
            pair("utf8", "café ☕"),
        ];
        let line = write_logfmt(pairs.clone());
        assert_eq!(parse_to_pairs(&line).unwrap(), pairs);
    }

    #[test]
    fn write_logfmt_accepts_owned_pairs() {
        let pairs = vec![("a".to_string(), "1".to_string())];
        assert_eq!(write_logfmt(pairs), "a=1");
    }

    #[test]
    fn lines_reader_yields_one_vec_per_line() {
        let data = b"at=info status=200\nat=warn status=503\n";
        let lines: Vec<_> = LogfmtLinesReader::new(BufReader::new(data.as_slice()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], vec![pair("at", "info"), pair("status", "200")]);
        assert_eq!(lines[1], vec![pair("at", "warn"), pair("status", "503")]);
    }

    #[test]
    fn lines_reader_skips_blank_lines_and_handles_crlf() {
        let data = b"a=1\r\n\r\n\nb=2";
        let lines: Vec<_> = parse_logfmt_lines(BufReader::new(data.as_slice()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(lines, vec![vec![pair("a", "1")], vec![pair("b", "2")]]);
    }

    #[test]
    fn lines_reader_reports_line_numbers() {
        let data = b"a=1\nmsg=\"unclosed\nc=3\n";
        let mut reader = LogfmtLinesReader::new(BufReader::new(data.as_slice()));
        assert_eq!(reader.line_number(), 0);
        reader.next().unwrap().unwrap();
        assert_eq!(reader.line_number(), 1);
        let err = reader.next().unwrap().unwrap_err();
        assert_eq!(err.line, 2);
        assert!(matches!(err.kind, LogfmtLinesErrorKind::Parse(_)));
        assert!(err.to_string().contains("line 2"));
    }

    #[test]
    fn lines_reader_empty_input() {
        let data: &[u8] = b"";
        let lines: Vec<_> = LogfmtLinesReader::new(BufReader::new(data))
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(lines.is_empty());
    }

    #[test]
    fn lines_reader_round_trips_written_lines() {
        let rows = vec![
            vec![pair("msg", "hello world"), pair("n", "1")],
            vec![pair("msg", "second\tline"), pair("n", "2")],
        ];
        let mut text = String::new();
        for row in &rows {
            text.push_str(&write_logfmt(row.clone()));
            text.push('\n');
        }
        let back: Vec<_> = parse_logfmt_lines(BufReader::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(back, rows);
    }
}
