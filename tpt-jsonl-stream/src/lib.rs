#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::fmt;
use std::io::{self, BufRead};

/// The kind of error that occurred while reading a JSON Lines stream.
#[derive(Debug)]
pub enum JsonlErrorKind {
    /// An I/O error from the underlying reader.
    Io(io::Error),
    /// A JSON parse error on a specific line.
    Json(serde_json::Error),
}

impl fmt::Display for JsonlErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Json(e) => write!(f, "JSON error: {}", e),
        }
    }
}

/// An error produced while reading or parsing a JSON Lines stream.
///
/// Includes the 1-based line number where the error occurred.
#[derive(Debug)]
pub struct JsonlError {
    /// The 1-based line number where the error occurred.
    pub line: u64,
    /// The underlying error kind.
    pub kind: JsonlErrorKind,
}

impl fmt::Display for JsonlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "jsonl error on line {}: {}", self.line, self.kind)
    }
}

impl std::error::Error for JsonlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            JsonlErrorKind::Io(e) => Some(e),
            JsonlErrorKind::Json(e) => Some(e),
        }
    }
}

/// A streaming JSON Lines reader.
///
/// Wraps any [`BufRead`] and yields one [`serde_json::Value`] per non-empty line.
/// Blank lines are silently skipped. Parse errors carry the line number.
///
/// # Example
///
/// ```
/// use tpt_jsonl_stream::JsonlReader;
/// use std::io::BufReader;
///
/// let data = b"{\"a\":1}\n{\"b\":2}\n";
/// let mut reader = JsonlReader::new(BufReader::new(data.as_slice()));
/// let first = reader.next().unwrap().unwrap();
/// assert_eq!(first["a"], 1);
/// ```
pub struct JsonlReader<R: BufRead> {
    reader: R,
    buf: String,
    line: u64,
}

impl<R: BufRead> JsonlReader<R> {
    /// Create a new `JsonlReader` wrapping the given buffered reader.
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

impl<R: BufRead> Iterator for JsonlReader<R> {
    type Item = Result<serde_json::Value, JsonlError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.buf.clear();
            match self.reader.read_line(&mut self.buf) {
                Err(e) => {
                    self.line += 1;
                    return Some(Err(JsonlError {
                        line: self.line,
                        kind: JsonlErrorKind::Io(e),
                    }));
                }
                Ok(0) => return None, // EOF
                Ok(_) => {
                    self.line += 1;
                    let trimmed = self.buf.trim();
                    if trimmed.is_empty() {
                        continue; // skip blank lines
                    }
                    #[cfg(feature = "simd")]
                    {
                        let mut bytes = trimmed.as_bytes().to_vec();
                        match simd_json::from_slice(&mut bytes) {
                            Ok(v) => return Some(Ok(v)),
                            Err(e) => {
                                return Some(Err(JsonlError {
                                    line: self.line,
                                    kind: JsonlErrorKind::Json(
                                        serde_json::from_str::<serde_json::Value>(trimmed)
                                            .unwrap_err(),
                                    ),
                                }));
                            }
                        }
                    }
                    #[cfg(not(feature = "simd"))]
                    match serde_json::from_str::<serde_json::Value>(trimmed) {
                        Ok(v) => return Some(Ok(v)),
                        Err(e) => {
                            return Some(Err(JsonlError {
                                line: self.line,
                                kind: JsonlErrorKind::Json(e),
                            }))
                        }
                    }
                }
            }
        }
    }
}

/// Create a [`JsonlReader`] from any [`BufRead`].
///
/// Convenience wrapper around [`JsonlReader::new`].
///
/// # Example
///
/// ```
/// use tpt_jsonl_stream::parse_jsonl;
/// use std::io::BufReader;
///
/// let data = b"{\"x\":1}\n\n{\"x\":2}\n";
/// let values: Vec<_> = parse_jsonl(BufReader::new(data.as_slice()))
///     .collect::<Result<_, _>>()
///     .unwrap();
/// assert_eq!(values.len(), 2);
/// ```
pub fn parse_jsonl<R: BufRead>(reader: R) -> JsonlReader<R> {
    JsonlReader::new(reader)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    fn read_all(data: &[u8]) -> Vec<serde_json::Value> {
        parse_jsonl(BufReader::new(data))
            .collect::<Result<_, _>>()
            .unwrap()
    }

    #[test]
    fn empty_input() {
        assert!(read_all(b"").is_empty());
    }

    #[test]
    fn single_line() {
        let vals = read_all(b"{\"k\":1}\n");
        assert_eq!(vals.len(), 1);
        assert_eq!(vals[0]["k"], 1);
    }

    #[test]
    fn multi_line() {
        let vals = read_all(b"{\"a\":1}\n{\"b\":2}\n{\"c\":3}\n");
        assert_eq!(vals.len(), 3);
    }

    #[test]
    fn blank_lines_skipped() {
        let vals = read_all(b"{\"a\":1}\n\n\n{\"b\":2}\n");
        assert_eq!(vals.len(), 2);
    }

    #[test]
    fn malformed_json_error_has_correct_line() {
        let data = b"{\"a\":1}\nNOT_JSON\n{\"c\":3}\n";
        let mut reader = parse_jsonl(BufReader::new(data.as_slice()));
        reader.next().unwrap().unwrap(); // line 1 ok
        let err = reader.next().unwrap().unwrap_err();
        assert_eq!(err.line, 2);
    }

    #[test]
    fn line_counter_exposed() {
        let data = b"{\"a\":1}\n{\"b\":2}\n";
        let mut reader = parse_jsonl(BufReader::new(data.as_slice()));
        assert_eq!(reader.line_number(), 0);
        reader.next();
        assert_eq!(reader.line_number(), 1);
    }

    #[test]
    fn no_trailing_newline() {
        let vals = read_all(b"{\"x\":42}");
        assert_eq!(vals.len(), 1);
        assert_eq!(vals[0]["x"], 42);
    }
}
