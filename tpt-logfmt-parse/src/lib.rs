#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

//! Zero-copy logfmt parser. See [`LogfmtParser`] for the streaming iterator API
//! and [`parse_to_map`] for the convenience owned-map API.

use std::collections::HashMap;
use std::fmt;

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

/// Zero-copy streaming logfmt parser.
///
/// Yields `(&str, &str)` key-value pairs borrowed directly from the input.
/// Keys without a `=` are yielded with an empty-string value.
///
/// # Example
///
/// ```
/// use tpt_logfmt_parse::LogfmtParser;
///
/// let input = r#"level=info msg="hello world" latency=42ms"#;
/// let pairs: Vec<_> = LogfmtParser::new(input).collect::<Result<_, _>>().unwrap();
/// assert_eq!(pairs[0], ("level", "info"));
/// assert_eq!(pairs[1], ("msg", "hello world"));
/// assert_eq!(pairs[2], ("latency", "42ms"));
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

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b' ' {
            self.pos += 1;
        }
    }

    /// Parse an unquoted token (key or bare value): everything up to `=`, `"`, or space.
    fn parse_bare(&mut self) -> &'a str {
        let start = self.pos;
        while self.pos < self.input.len() {
            match self.input.as_bytes()[self.pos] {
                b'=' | b' ' | b'"' => break,
                _ => self.pos += 1,
            }
        }
        &self.input[start..self.pos]
    }

    /// Parse a quoted string value, returning the content between the quotes.
    /// Escape sequences `\"` and `\\` are supported; the returned slice is from
    /// the raw input (no allocation) for non-escaped strings, but we fall back
    /// to returning an empty slice on escape sequences to avoid allocation.
    ///
    /// For simplicity and true zero-copy, escaped content is returned as a raw
    /// slice that still contains backslashes — callers needing unescaped content
    /// should use [`parse_to_map`] which allocates and resolves escapes.
    fn parse_quoted(&mut self) -> Result<&'a str, LogfmtError> {
        debug_assert_eq!(self.input.as_bytes()[self.pos], b'"');
        self.pos += 1; // consume opening quote
        let start = self.pos;
        loop {
            if self.pos >= self.input.len() {
                return Err(LogfmtError {
                    position: self.pos,
                    message: "unterminated quoted string",
                });
            }
            match self.input.as_bytes()[self.pos] {
                b'"' => {
                    let slice = &self.input[start..self.pos];
                    self.pos += 1; // consume closing quote
                    return Ok(slice);
                }
                b'\\' => {
                    self.pos += 1; // skip escape char
                    if self.pos >= self.input.len() {
                        return Err(LogfmtError {
                            position: self.pos,
                            message: "unterminated escape sequence",
                        });
                    }
                    self.pos += 1; // skip escaped char
                }
                _ => self.pos += 1,
            }
        }
    }
}

impl<'a> Iterator for LogfmtParser<'a> {
    type Item = Result<(&'a str, &'a str), LogfmtError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();
        if self.pos >= self.input.len() {
            return None;
        }

        let key = self.parse_bare();
        if key.is_empty() {
            return Some(Err(LogfmtError {
                position: self.pos,
                message: "expected key",
            }));
        }

        // No `=` — bare key with empty value
        if self.pos >= self.input.len() || self.input.as_bytes()[self.pos] != b'=' {
            return Some(Ok((key, "")));
        }

        self.pos += 1; // consume `=`

        // Value: quoted or bare
        if self.pos < self.input.len() && self.input.as_bytes()[self.pos] == b'"' {
            Some(self.parse_quoted().map(|v| (key, v)))
        } else {
            Some(Ok((key, self.parse_bare())))
        }
    }
}

/// Parse a logfmt string into an owned `HashMap<String, String>`.
///
/// Escape sequences inside quoted values (`\"`, `\\`) are resolved.
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
    let mut pos = 0usize;

    let skip_ws = |p: &mut usize| {
        while *p < input.len() && input.as_bytes()[*p] == b' ' {
            *p += 1;
        }
    };

    let parse_bare_owned = |p: &mut usize| -> String {
        let start = *p;
        while *p < input.len() {
            match input.as_bytes()[*p] {
                b'=' | b' ' | b'"' => break,
                _ => *p += 1,
            }
        }
        input[start..*p].to_owned()
    };

    loop {
        skip_ws(&mut pos);
        if pos >= input.len() {
            break;
        }

        // Parse key (always bare)
        let key = parse_bare_owned(&mut pos);
        if key.is_empty() {
            return Err(LogfmtError {
                position: pos,
                message: "expected key",
            });
        }

        if pos >= input.len() || input.as_bytes()[pos] != b'=' {
            map.insert(key, String::new());
            continue;
        }
        pos += 1; // consume `=`

        let value = if pos < input.len() && input.as_bytes()[pos] == b'"' {
            pos += 1; // consume opening quote
            let mut val = String::new();
            loop {
                if pos >= input.len() {
                    return Err(LogfmtError {
                        position: pos,
                        message: "unterminated quoted string",
                    });
                }
                match input.as_bytes()[pos] {
                    b'"' => {
                        pos += 1;
                        break;
                    }
                    b'\\' => {
                        pos += 1;
                        if pos >= input.len() {
                            return Err(LogfmtError {
                                position: pos,
                                message: "unterminated escape sequence",
                            });
                        }
                        match input.as_bytes()[pos] {
                            b'"' => {
                                val.push('"');
                                pos += 1;
                            }
                            b'\\' => {
                                val.push('\\');
                                pos += 1;
                            }
                            b'n' => {
                                val.push('\n');
                                pos += 1;
                            }
                            b't' => {
                                val.push('\t');
                                pos += 1;
                            }
                            other => {
                                val.push(other as char);
                                pos += 1;
                            }
                        }
                    }
                    b => {
                        val.push(b as char);
                        pos += 1;
                    }
                }
            }
            val
        } else {
            parse_bare_owned(&mut pos)
        };

        map.insert(key, value);
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_pairs() {
        let pairs: Vec<_> = LogfmtParser::new("a=1 b=2")
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(pairs, vec![("a", "1"), ("b", "2")]);
    }

    #[test]
    fn quoted_value() {
        let pairs: Vec<_> = LogfmtParser::new(r#"msg="hello world""#)
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(pairs[0].0, "msg");
        assert_eq!(pairs[0].1, "hello world");
    }

    #[test]
    fn bare_key_no_value() {
        let pairs: Vec<_> = LogfmtParser::new("flag").collect::<Result<_, _>>().unwrap();
        assert_eq!(pairs, vec![("flag", "")]);
    }

    #[test]
    fn mixed() {
        let pairs: Vec<_> = LogfmtParser::new(r#"level=info flag msg="ok" count=3"#)
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(pairs[0], ("level", "info"));
        assert_eq!(pairs[1], ("flag", ""));
        assert_eq!(pairs[2], ("msg", "ok"));
        assert_eq!(pairs[3], ("count", "3"));
    }

    #[test]
    fn empty_input() {
        let pairs: Vec<_> = LogfmtParser::new("").collect::<Result<_, _>>().unwrap();
        assert!(pairs.is_empty());
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
}
