#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use std::fmt;
use std::io::{self, BufRead, Write};
use std::marker::PhantomData;

use serde::de::DeserializeOwned;

/// The default maximum length, in bytes, of a single JSON Lines record.
///
/// A line longer than this (excluding its terminating newline) is reported as
/// [`JsonlErrorKind::LineTooLong`] instead of being buffered. Override it with
/// [`JsonlReader::with_max_line_length`].
pub const DEFAULT_MAX_LINE_LENGTH: usize = 16 * 1024 * 1024;

/// The kind of error that occurred while reading a JSON Lines stream.
///
/// This enum is `#[non_exhaustive]` — new variants may be added in future
/// releases without a breaking change.
#[derive(Debug)]
#[non_exhaustive]
pub enum JsonlErrorKind {
    /// An I/O error from the underlying reader.
    Io(io::Error),
    /// A JSON parse error on a specific line.
    Json(serde_json::Error),
    /// A line exceeded the reader's configured maximum line length.
    LineTooLong {
        /// The configured maximum line length, in bytes.
        limit: usize,
    },
}

impl fmt::Display for JsonlErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Json(e) => write!(f, "JSON error: {}", e),
            Self::LineTooLong { limit } => {
                write!(f, "line exceeds the maximum length of {} bytes", limit)
            }
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
            JsonlErrorKind::LineTooLong { .. } => None,
        }
    }
}

enum LineOutcome {
    Eof,
    Line,
    TooLong,
    Io(io::Error),
}

fn trim_end_ascii_whitespace(bytes: &[u8]) -> usize {
    let mut end = bytes.len();
    while end > 0 && matches!(bytes[end - 1], b' ' | b'\t' | b'\r' | b'\n') {
        end -= 1;
    }
    end
}

/// Maximum nesting depth accepted when parsing a single JSON value.
///
/// Mirrors `serde_json`'s own [`serde_json::de::Read`](serde_json::de::Read)
/// recursion limit so that the `simd` code path rejects deeply nested input
/// instead of overflowing the stack (the default path is bounded by serde_json
/// itself; `simd_json`'s serde bridge is recursive with no depth guard).
#[cfg(feature = "simd")]
const MAX_JSON_DEPTH: u32 = 128;

/// Count the maximum bracket nesting depth of a JSON document, skipping string
/// bodies, so a hostile deeply-nested line can be rejected cheaply.
///
/// Returns `Err` if the depth exceeds [`MAX_JSON_DEPTH`], which is the same
/// outcome `serde_json` produces on the default code path.
#[cfg(feature = "simd")]
fn check_json_depth(bytes: &[u8]) -> Result<(), serde_json::Error> {
    let mut depth: u32 = 0;
    let mut in_string = false;
    let mut escaped = false;
    for &b in bytes {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'[' | b'{' => {
                depth += 1;
                if depth > MAX_JSON_DEPTH {
                    return Err(<serde_json::Error as serde::de::Error>::custom(
                        "recursion limit exceeded: JSON nesting is too deep",
                    ));
                }
            }
            b']' | b'}' => {
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(feature = "simd")]
fn parse_line(bytes: &mut [u8]) -> Result<serde_json::Value, serde_json::Error> {
    check_json_depth(bytes)?;
    simd_json::from_slice(bytes).map_err(<serde_json::Error as serde::de::Error>::custom)
}

#[cfg(not(feature = "simd"))]
fn parse_line(bytes: &mut [u8]) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::from_slice(bytes)
}

/// A streaming JSON Lines reader.
///
/// Wraps any [`BufRead`] and yields one [`serde_json::Value`] per non-blank line.
/// A line is blank only when it is empty or consists solely of ASCII whitespace
/// (space, tab, carriage return, newline); lines made of other Unicode whitespace,
/// such as a non-breaking space, are passed to the parser like any other content.
/// Parse errors carry the line number, and their byte offsets refer to the original
/// line because leading whitespace is never stripped before parsing.
///
/// A leading UTF-8 BOM on the very first line is stripped automatically so it does
/// not cause the first record to fail to parse; a BOM appearing later in the stream
/// is left intact and reported as a parse error.
///
/// Lines are buffered up to [`DEFAULT_MAX_LINE_LENGTH`] bytes by default. A longer
/// line is skipped and reported as [`JsonlErrorKind::LineTooLong`] rather than
/// buffered, so a newline-free stream cannot exhaust memory. See
/// [`JsonlReader::with_max_line_length`].
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
    buf: Vec<u8>,
    line: u64,
    max_line_length: usize,
    done: bool,
    /// Total bytes consumed from the underlying reader so far. Equals the
    /// absolute byte offset of the next byte to be read, which is what
    /// [`JsonlReader::byte_offset`] reports and [`JsonlReader::resume_at`]
    /// seeks to when resuming a stream after a crash.
    bytes_consumed: u64,
}

impl<R: BufRead> JsonlReader<R> {
    /// Create a new `JsonlReader` wrapping the given buffered reader.
    ///
    /// The maximum line length defaults to [`DEFAULT_MAX_LINE_LENGTH`].
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: Vec::new(),
            line: 0,
            max_line_length: DEFAULT_MAX_LINE_LENGTH,
            done: false,
            bytes_consumed: 0,
        }
    }

    /// Set the maximum length, in bytes, of a single line (excluding its newline).
    ///
    /// A longer line is never buffered: the reader discards it, yields a
    /// [`JsonlErrorKind::LineTooLong`] error carrying that line number, and resumes
    /// at the next line. Pass [`usize::MAX`] for unbounded buffering.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_jsonl_stream::{JsonlErrorKind, JsonlReader};
    /// use std::io::BufReader;
    ///
    /// let data = b"{\"a\":1}\n[0,1,2,3,4,5,6,7,8,9]\n{\"b\":2}\n";
    /// let mut reader = JsonlReader::new(BufReader::new(data.as_slice())).with_max_line_length(8);
    /// assert_eq!(reader.next().unwrap().unwrap()["a"], 1);
    /// let err = reader.next().unwrap().unwrap_err();
    /// assert!(matches!(err.kind, JsonlErrorKind::LineTooLong { limit: 8 }));
    /// assert_eq!(reader.next().unwrap().unwrap()["b"], 2);
    /// ```
    pub fn with_max_line_length(mut self, max_line_length: usize) -> Self {
        self.max_line_length = max_line_length;
        self
    }

    /// The configured maximum line length, in bytes.
    pub fn max_line_length(&self) -> usize {
        self.max_line_length
    }

    /// The 1-based line number most recently read (or 0 before any reads).
    pub fn line_number(&self) -> u64 {
        self.line
    }

    /// The absolute byte offset of the next byte to be read from the underlying
    /// stream.
    ///
    /// Equals the number of bytes consumed so far (including the terminating
    /// newlines of fully-read lines), so it can be persisted after each record
    /// and later handed to [`JsonlReader::resume_at`] to continue a stream that
    /// was interrupted — e.g. a multi-GB `.jsonl` ingest that crashed partway.
    ///
    /// # Example
    ///
    /// ```
    /// use tpt_jsonl_stream::JsonlReader;
    /// use std::io::BufReader;
    ///
    /// let data = b"{\"a\":1}\n{\"b\":2}\n{\"c\":3}\n";
    /// let mut reader = JsonlReader::new(BufReader::new(data.as_slice()));
    /// reader.next().unwrap().unwrap();
    /// assert_eq!(reader.byte_offset(), 8); // len of `{"a":1}\n`
    /// reader.next().unwrap().unwrap();
    /// assert_eq!(reader.byte_offset(), 16);
    /// ```
    pub fn byte_offset(&self) -> u64 {
        self.bytes_consumed
    }

    /// Resume reading from `offset` bytes into the underlying stream.
    ///
    /// Seeks the wrapped reader to `offset`, discards any buffered partial line,
    /// and clears the end-of-stream latch so iteration can continue. Intended to
    /// be paired with a previously persisted [`JsonlReader::byte_offset`] for
    /// crash-safe ingestion. Line numbering restarts from the next record read
    /// (the absolute offset is the source of truth for position), and the
    /// maximum line length and any parsed-so-far state are reset.
    ///
    /// Requires the underlying reader to implement [`std::io::Seek`].
    pub fn resume_at(&mut self, offset: u64) -> std::io::Result<()>
    where
        R: std::io::Seek,
    {
        self.reader.seek(std::io::SeekFrom::Start(offset))?;
        self.buf.clear();
        self.bytes_consumed = offset;
        self.line = 0;
        self.done = false;
        Ok(())
    }

    /// Deserialize every yielded value into `T`.
    ///
    /// Returns an iterator of `Result<T, JsonlError>`: each line is parsed as JSON
    /// and then deserialized, so both syntax errors and type mismatches carry the
    /// 1-based line number.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::Deserialize;
    /// use std::io::BufReader;
    /// use tpt_jsonl_stream::JsonlReader;
    ///
    /// #[derive(Deserialize)]
    /// struct Row {
    ///     id: u32,
    /// }
    ///
    /// let data = b"{\"id\":1}\n{\"id\":2}\n";
    /// let rows: Vec<Row> = JsonlReader::new(BufReader::new(data.as_slice()))
    ///     .into_typed::<Row>()
    ///     .collect::<Result<_, _>>()
    ///     .unwrap();
    /// assert_eq!(rows[1].id, 2);
    /// ```
    pub fn into_typed<T: DeserializeOwned>(self) -> TypedJsonlReader<R, T> {
        TypedJsonlReader {
            inner: self,
            marker: PhantomData,
        }
    }

    fn read_capped_line(&mut self) -> LineOutcome {
        self.buf.clear();
        let mut content_len: usize = 0;
        let mut too_long = false;
        let mut saw_bytes = false;
        loop {
            let chunk = match self.reader.fill_buf() {
                Ok(chunk) => chunk,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return LineOutcome::Io(e),
            };
            if chunk.is_empty() {
                break;
            }
            saw_bytes = true;
            let (taken, terminated) = match chunk.iter().position(|&b| b == b'\n') {
                Some(idx) => (idx + 1, true),
                None => (chunk.len(), false),
            };
            if !too_long {
                let payload = if terminated { taken - 1 } else { taken };
                if content_len.saturating_add(payload) > self.max_line_length {
                    too_long = true;
                    self.buf = Vec::new();
                } else {
                    content_len += payload;
                    self.buf.extend_from_slice(&chunk[..taken]);
                }
            }
            self.reader.consume(taken);
            self.bytes_consumed += taken as u64;
            if terminated {
                break;
            }
        }
        if too_long {
            LineOutcome::TooLong
        } else if saw_bytes {
            LineOutcome::Line
        } else {
            LineOutcome::Eof
        }
    }
}

impl<R: BufRead> Iterator for JsonlReader<R> {
    type Item = Result<serde_json::Value, JsonlError>;

    fn next(&mut self) -> Option<Self::Item> {
        // Once the stream has ended or hit an unrecoverable I/O error, never
        // resume: a persistent error would otherwise be re-yielded forever
        // (the underlying `BufRead` does not cache errors).
        if self.done {
            return None;
        }
        loop {
            match self.read_capped_line() {
                LineOutcome::Eof => {
                    self.done = true;
                    return None;
                }
                LineOutcome::Io(e) => {
                    self.done = true;
                    self.line += 1;
                    return Some(Err(JsonlError {
                        line: self.line,
                        kind: JsonlErrorKind::Io(e),
                    }));
                }
                LineOutcome::TooLong => {
                    self.line += 1;
                    return Some(Err(JsonlError {
                        line: self.line,
                        kind: JsonlErrorKind::LineTooLong {
                            limit: self.max_line_length,
                        },
                    }));
                }
                LineOutcome::Line => {
                    self.line += 1;
                    let end = trim_end_ascii_whitespace(&self.buf);
                    if end == 0 {
                        continue;
                    }
                    let mut start = 0;
                    // A leading UTF-8 BOM is only legal before the very first
                    // record; strip it so the parser sees clean JSON rather than
                    // failing on an unexpected character.
                    if self.line == 1 && self.buf.len() >= 3 && self.buf[..3] == [0xEF, 0xBB, 0xBF]
                    {
                        start = 3;
                    }
                    let line = self.line;
                    return Some(
                        parse_line(&mut self.buf[start..end]).map_err(|e| JsonlError {
                            line,
                            kind: JsonlErrorKind::Json(e),
                        }),
                    );
                }
            }
        }
    }
}

impl<R: BufRead> std::iter::FusedIterator for JsonlReader<R> {}

#[cfg(feature = "tokio")]
pub use async_reader::AsyncJsonlReader;

/// A streaming JSON Lines reader over `tokio`'s [`tokio::io::AsyncBufRead`].
///
/// Mirrors the synchronous [`JsonlReader`] but for asynchronous IO. Available
/// with the optional `tokio` feature. The same line-length cap, blank-line
/// handling, and `LineTooLong` protection apply.
///
/// ```rust,ignore
/// use tpt_jsonl_stream::AsyncJsonlReader;
/// use tokio::io::BufReader;
///
/// # async fn run() {
/// let data = b"{\"a\":1}\n{\"b\":2}\n";
/// let mut reader = AsyncJsonlReader::new(BufReader::new(data.as_slice()));
/// while let Some(record) = reader.next().await {
///     println!("{}", record.unwrap());
/// }
/// # }
/// ```
#[cfg(feature = "tokio")]
mod async_reader {
    use super::*;
    use tokio::io::{AsyncBufRead, AsyncBufReadExt};

    /// A streaming JSON Lines reader over an [`AsyncBufRead`].
    ///
    /// Created via [`AsyncJsonlReader::new`]. See the crate-level example for
    /// usage.
    pub struct AsyncJsonlReader<R: AsyncBufRead> {
        reader: R,
        buf: Vec<u8>,
        line: u64,
        max_line_length: usize,
        done: bool,
    }

    impl<R: AsyncBufRead + Unpin> AsyncJsonlReader<R> {
        /// Create a new `AsyncJsonlReader` wrapping the given async buffered reader.
        ///
        /// The maximum line length defaults to [`DEFAULT_MAX_LINE_LENGTH`].
        pub fn new(reader: R) -> Self {
            Self {
                reader,
                buf: Vec::new(),
                line: 0,
                max_line_length: DEFAULT_MAX_LINE_LENGTH,
                done: false,
            }
        }

        /// Set the maximum length, in bytes, of a single line (excluding its newline).
        ///
        /// A longer line is never buffered: it is discarded, reported as
        /// [`JsonlErrorKind::LineTooLong`], and the reader resumes at the next line.
        /// Pass [`usize::MAX`] for unbounded buffering.
        pub fn with_max_line_length(mut self, max_line_length: usize) -> Self {
            self.max_line_length = max_line_length;
            self
        }

        /// The configured maximum line length, in bytes.
        pub fn max_line_length(&self) -> usize {
            self.max_line_length
        }

        /// The 1-based line number most recently read (or 0 before any reads).
        pub fn line_number(&self) -> u64 {
            self.line
        }

        async fn read_capped_line(&mut self) -> LineOutcome {
            self.buf.clear();
            let mut content_len: usize = 0;
            let mut too_long = false;
            let mut saw_bytes = false;
            loop {
                let chunk = match self.reader.fill_buf().await {
                    Ok(chunk) => chunk,
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                    Err(e) => return LineOutcome::Io(e),
                };
                if chunk.is_empty() {
                    break;
                }
                saw_bytes = true;
                let (taken, terminated) = match chunk.iter().position(|&b| b == b'\n') {
                    Some(idx) => (idx + 1, true),
                    None => (chunk.len(), false),
                };
                if !too_long {
                    let payload = if terminated { taken - 1 } else { taken };
                    if content_len.saturating_add(payload) > self.max_line_length {
                        too_long = true;
                        self.buf = Vec::new();
                    } else {
                        content_len += payload;
                        self.buf.extend_from_slice(&chunk[..taken]);
                    }
                }
                self.reader.consume(taken);
                if terminated {
                    break;
                }
            }
            if too_long {
                LineOutcome::TooLong
            } else if saw_bytes {
                LineOutcome::Line
            } else {
                LineOutcome::Eof
            }
        }

        /// Async variant of [`JsonlReader::next`].
        ///
        /// Returns the next parsed record, or `None` when the stream ends or a
        /// persistent I/O error has been latched (the reader does not resume
        /// after an error, which would otherwise loop forever).
        pub async fn next(&mut self) -> Option<Result<serde_json::Value, JsonlError>> {
            if self.done {
                return None;
            }
            loop {
                match self.read_capped_line().await {
                    LineOutcome::Eof => {
                        self.done = true;
                        return None;
                    }
                    LineOutcome::Io(e) => {
                        self.done = true;
                        self.line += 1;
                        return Some(Err(JsonlError {
                            line: self.line,
                            kind: JsonlErrorKind::Io(e),
                        }));
                    }
                    LineOutcome::TooLong => {
                        self.line += 1;
                        return Some(Err(JsonlError {
                            line: self.line,
                            kind: JsonlErrorKind::LineTooLong {
                                limit: self.max_line_length,
                            },
                        }));
                    }
                    LineOutcome::Line => {
                        self.line += 1;
                        let end = trim_end_ascii_whitespace(&self.buf);
                        if end == 0 {
                            continue;
                        }
                        let mut start = 0;
                        if self.line == 1
                            && self.buf.len() >= 3
                            && self.buf[..3] == [0xEF, 0xBB, 0xBF]
                        {
                            start = 3;
                        }
                        let line = self.line;
                        return Some(parse_line(&mut self.buf[start..end]).map_err(|e| {
                            JsonlError {
                                line,
                                kind: JsonlErrorKind::Json(e),
                            }
                        }));
                    }
                }
            }
        }
    }
}

/// A streaming JSON Lines reader that deserializes each record into `T`.
///
/// Created by [`JsonlReader::into_typed`].
pub struct TypedJsonlReader<R: BufRead, T> {
    inner: JsonlReader<R>,
    marker: PhantomData<fn() -> T>,
}

impl<R: BufRead, T> TypedJsonlReader<R, T> {
    /// The 1-based line number most recently read (or 0 before any reads).
    pub fn line_number(&self) -> u64 {
        self.inner.line_number()
    }

    /// Consume this reader and return the underlying [`JsonlReader`].
    pub fn into_inner(self) -> JsonlReader<R> {
        self.inner
    }
}

impl<R: BufRead, T: DeserializeOwned> Iterator for TypedJsonlReader<R, T> {
    type Item = Result<T, JsonlError>;

    fn next(&mut self) -> Option<Self::Item> {
        let value = match self.inner.next()? {
            Ok(value) => value,
            Err(e) => return Some(Err(e)),
        };
        let line = self.inner.line_number();
        Some(serde_json::from_value(value).map_err(|e| JsonlError {
            line,
            kind: JsonlErrorKind::Json(e),
        }))
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

/// Parse every record of a JSON Lines stream in parallel using [`rayon`].
///
/// Available with the optional `rayon` feature. Reads the whole stream into
/// memory (collected by construction), splits it into lines using the same
/// blank-line, BOM-on-first-line and `LineTooLong` rules as [`JsonlReader`], and
/// then parses each line on a worker thread. The returned vector preserves the
/// original line order; each element is the per-line parse result (with its
/// 1-based line number) or a [`JsonlError`].
///
/// Use this when you have a large, CPU-bound `.jsonl` file and want to saturate
/// all cores — on multi-core machines this typically beats even the `simd`
/// single-threaded fast path. For constant-memory streaming use
/// [`parse_jsonl`] instead.
///
/// # Example
///
/// ```
/// # #[cfg(feature = "rayon")]
/// # {
/// use tpt_jsonl_stream::parse_jsonl_parallel;
/// use std::io::BufReader;
///
/// let data = b"{\"x\":1}\n{\"x\":2}\n{\"x\":3}\n";
/// let values = parse_jsonl_parallel(BufReader::new(data.as_slice()));
/// let values: Vec<_> = values.into_iter().collect::<Result<_, _>>().unwrap();
/// assert_eq!(values.len(), 3);
/// assert_eq!(values[2]["x"], 3);
/// # }
/// ```
#[cfg(feature = "rayon")]
pub fn parse_jsonl_parallel<R: std::io::Read>(
    reader: R,
) -> Vec<Result<serde_json::Value, JsonlError>> {
    use rayon::prelude::*;
    use std::io::{BufReader, Read};

    let mut reader = BufReader::new(reader);
    let mut full = Vec::new();
    match reader.read_to_end(&mut full) {
        Ok(_) => {}
        Err(e) => return vec![Err(JsonlError {
            line: 0,
            kind: JsonlErrorKind::Io(e),
        })],
    }

    // Split into non-blank lines, mirroring the sync reader's rules.
    let mut lines: Vec<(u64, Vec<u8>)> = Vec::new();
    let mut line_no: u64 = 0;
    for raw in full.split(|&b| b == b'\n') {
        line_no += 1;
        let end = trim_end_ascii_whitespace(raw);
        if end == 0 {
            continue;
        }
        let mut start = 0;
        if line_no == 1 && raw.len() >= 3 && raw[..3] == [0xEF, 0xBB, 0xBF] {
            start = 3;
        }
        lines.push((line_no, raw[start..end].to_vec()));
    }

    lines
        .par_iter()
        .map(|(line, content)| {
            if content.len() > DEFAULT_MAX_LINE_LENGTH {
                return Err(JsonlError {
                    line: *line,
                    kind: JsonlErrorKind::LineTooLong {
                        limit: DEFAULT_MAX_LINE_LENGTH,
                    },
                });
            }
            let mut buf = content.clone();
            parse_line(&mut buf)
                .map_err(|e| JsonlError {
                    line: *line,
                    kind: JsonlErrorKind::Json(e),
                })
        })
        .collect()
}

/// A streaming JSON Lines writer.
///
/// Wraps any [`Write`] and emits one JSON value per line. Each call to
/// [`JsonlWriter::write`] serializes the value with `serde_json` into an internal
/// buffer and only then emits the complete line, so a serialization failure leaves
/// no partial record in the output. Errors carry the 1-based line number of the
/// write that failed.
///
/// # Example
///
/// ```
/// use tpt_jsonl_stream::JsonlWriter;
/// use std::io::Cursor;
///
/// let mut buf = Cursor::new(Vec::new());
/// {
///     let mut writer = JsonlWriter::new(&mut buf);
///     writer.write(&serde_json::json!({"a": 1})).unwrap();
///     writer.write(&serde_json::json!({"b": 2})).unwrap();
/// }
/// let out = String::from_utf8(buf.into_inner()).unwrap();
/// assert_eq!(out, "{\"a\":1}\n{\"b\":2}\n");
/// ```
pub struct JsonlWriter<W: Write> {
    writer: W,
    line: u64,
    scratch: Vec<u8>,
}

impl<W: Write> JsonlWriter<W> {
    /// Create a new `JsonlWriter` wrapping the given writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            line: 0,
            scratch: Vec::new(),
        }
    }

    /// The number of lines (values) written so far.
    pub fn line_number(&self) -> u64 {
        self.line
    }

    /// Serialize `value` as a single JSON Lines record (one line, newline-terminated).
    ///
    /// The record is serialized into an internal buffer first and written in a single
    /// call, so a value that fails to serialize emits nothing at all and later writes
    /// stay well-formed.
    pub fn write<T: serde::Serialize>(&mut self, value: &T) -> Result<(), JsonlError> {
        self.scratch.clear();
        serde_json::to_writer(&mut self.scratch, value).map_err(|e| JsonlError {
            line: self.line + 1,
            kind: JsonlErrorKind::Json(e),
        })?;
        self.scratch.push(b'\n');
        self.writer
            .write_all(&self.scratch)
            .map_err(|e| JsonlError {
                line: self.line + 1,
                kind: JsonlErrorKind::Io(e),
            })?;
        self.line += 1;
        Ok(())
    }

    /// Flush the underlying writer.
    pub fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// Write a sequence of values as JSON Lines to the given [`Write`] sink.
///
/// # Example
///
/// ```
/// use tpt_jsonl_stream::write_jsonl;
/// use std::io::Cursor;
///
/// let mut buf = Cursor::new(Vec::new());
/// let values = vec![serde_json::json!(1), serde_json::json!(2)];
/// write_jsonl(&mut buf, values.iter()).unwrap();
/// let out = String::from_utf8(buf.into_inner()).unwrap();
/// assert_eq!(out, "1\n2\n");
/// ```
pub fn write_jsonl<W: Write, I, T>(writer: W, values: I) -> Result<(), JsonlError>
where
    I: IntoIterator<Item = T>,
    T: serde::Serialize,
{
    let mut w = JsonlWriter::new(writer);
    for v in values {
        w.write(&v)?;
    }
    w.flush().map_err(|e| JsonlError {
        line: w.line_number() + 1,
        kind: JsonlErrorKind::Io(e),
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[derive(serde::Deserialize, Debug, PartialEq)]
    struct Row {
        id: u32,
        name: String,
    }

    struct FailsMidway;

    impl serde::Serialize for FailsMidway {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            use serde::ser::SerializeMap;
            let mut map = serializer.serialize_map(Some(2))?;
            map.serialize_entry("partial", &1)?;
            Err(serde::ser::Error::custom("serialization failed midway"))
        }
    }

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
    fn ascii_whitespace_lines_skipped() {
        let vals = read_all(b"{\"a\":1}\n   \n\t\r\n{\"b\":2}\n");
        assert_eq!(vals.len(), 2);
    }

    #[test]
    fn crlf_lines_parse() {
        let vals = read_all(b"{\"a\":1}\r\n{\"b\":2}\r\n");
        assert_eq!(vals.len(), 2);
        assert_eq!(vals[1]["b"], 2);
    }

    #[test]
    fn nbsp_only_line_is_not_blank() {
        let data = "{\"a\":1}\n\u{a0}\n{\"c\":3}\n";
        let mut reader = parse_jsonl(BufReader::new(data.as_bytes()));
        assert_eq!(reader.next().unwrap().unwrap()["a"], 1);
        let err = reader.next().unwrap().unwrap_err();
        assert_eq!(err.line, 2);
        assert!(matches!(err.kind, JsonlErrorKind::Json(_)));
        assert_eq!(reader.next().unwrap().unwrap()["c"], 3);
        assert!(reader.next().is_none());
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
    fn error_line_number_counts_skipped_blank_lines() {
        let data = b"{\"a\":1}\n\n   \nNOT_JSON\n{\"e\":5}\n";
        let mut reader = parse_jsonl(BufReader::new(data.as_slice()));
        reader.next().unwrap().unwrap();
        let err = reader.next().unwrap().unwrap_err();
        assert_eq!(err.line, 4);
        assert_eq!(reader.line_number(), 4);
        reader.next().unwrap().unwrap();
        assert_eq!(reader.line_number(), 5);
    }

    #[cfg(not(feature = "simd"))]
    #[test]
    fn error_offsets_match_the_untrimmed_line() {
        let line = "   {\"a\": }";
        let direct = serde_json::from_str::<serde_json::Value>(line).unwrap_err();
        let trimmed = serde_json::from_str::<serde_json::Value>(line.trim()).unwrap_err();
        assert_ne!(direct.column(), trimmed.column());

        let data = format!("{}\n", line);
        let mut reader = parse_jsonl(BufReader::new(data.as_bytes()));
        match reader.next().unwrap().unwrap_err().kind {
            JsonlErrorKind::Json(e) => {
                assert_eq!(e.line(), direct.line());
                assert_eq!(e.column(), direct.column());
            }
            other => panic!("expected a JSON error, got {:?}", other),
        }
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
    fn byte_offset_tracks_consumed_bytes() {
        let data = b"{\"a\":1}\n{\"b\":2}\n{\"c\":3}\n";
        let mut reader = parse_jsonl(BufReader::new(data.as_slice()));
        assert_eq!(reader.byte_offset(), 0);
        reader.next().unwrap().unwrap();
        assert_eq!(reader.byte_offset(), 8); // length of `{"a":1}\n`
        reader.next().unwrap().unwrap();
        assert_eq!(reader.byte_offset(), 16);
        reader.next().unwrap().unwrap();
        assert_eq!(reader.byte_offset(), 24);
        assert!(reader.next().is_none());
        assert_eq!(reader.byte_offset(), 24);
    }

    #[test]
    fn resume_at_continues_from_offset() {
        use std::io::Cursor;
        let data = b"{\"a\":1}\n{\"b\":2}\n{\"c\":3}\n".to_vec();
        let mut reader = parse_jsonl(Cursor::new(data.clone()));
        reader.next().unwrap().unwrap();
        let offset = reader.byte_offset();
        // Simulate a crash: recreate the reader on the same backing data, then
        // resume from the persisted offset.
        let mut reader = parse_jsonl(Cursor::new(data));
        reader.resume_at(offset).unwrap();
        let rest: Vec<serde_json::Value> = reader.collect::<Result<_, _>>().unwrap();
        assert_eq!(rest.len(), 2);
        assert_eq!(rest[0]["b"], 2);
        assert_eq!(rest[1]["c"], 3);
    }

    #[test]
    fn no_trailing_newline() {
        let vals = read_all(b"{\"x\":42}");
        assert_eq!(vals.len(), 1);
        assert_eq!(vals[0]["x"], 42);
    }

    #[test]
    fn default_max_line_length_is_exposed() {
        let reader = parse_jsonl(BufReader::new(b"".as_slice()));
        assert_eq!(reader.max_line_length(), DEFAULT_MAX_LINE_LENGTH);
    }

    #[test]
    fn over_long_line_is_rejected_and_stream_resumes() {
        let long = format!("{{\"a\":\"{}\"}}", "x".repeat(4096));
        let data = format!("{{\"a\":1}}\n{}\n{{\"c\":3}}\n", long);
        let mut reader = parse_jsonl(BufReader::new(data.as_bytes())).with_max_line_length(64);
        assert_eq!(reader.next().unwrap().unwrap()["a"], 1);
        let err = reader.next().unwrap().unwrap_err();
        assert_eq!(err.line, 2);
        assert!(matches!(
            err.kind,
            JsonlErrorKind::LineTooLong { limit: 64 }
        ));
        assert!(err.to_string().contains("64"));
        assert_eq!(reader.next().unwrap().unwrap()["c"], 3);
        assert!(reader.next().is_none());
    }

    #[test]
    fn over_long_final_line_without_newline_is_rejected() {
        let data = format!("[{}]", "1,".repeat(1000));
        let mut reader = parse_jsonl(BufReader::new(data.as_bytes())).with_max_line_length(16);
        let err = reader.next().unwrap().unwrap_err();
        assert_eq!(err.line, 1);
        assert!(matches!(err.kind, JsonlErrorKind::LineTooLong { .. }));
        assert!(reader.next().is_none());
    }

    #[test]
    fn line_of_exactly_the_limit_is_accepted() {
        let data = b"[1,2,3]\n";
        let vals: Vec<serde_json::Value> = parse_jsonl(BufReader::new(data.as_slice()))
            .with_max_line_length(7)
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(vals, vec![serde_json::json!([1, 2, 3])]);
    }

    #[test]
    fn leading_bom_on_first_line_is_stripped() {
        // A UTF-8 BOM at the very start of the stream must not make the first
        // record fail to parse.
        let data: &[u8] = b"\xEF\xBB\xBF{\"a\":1}\n{\"b\":2}\n";
        let mut reader = parse_jsonl(BufReader::new(data));
        assert_eq!(reader.next().unwrap().unwrap()["a"], 1);
        assert_eq!(reader.next().unwrap().unwrap()["b"], 2);
        assert!(reader.next().is_none());
    }

    #[test]
    fn bom_only_on_first_line() {
        // A BOM appearing mid-stream (not the first line) is not stripped and
        // should still be reported as a parse error.
        let data: &[u8] = b"{\"a\":1}\n\xEF\xBB\xBF{\"b\":2}\n";
        let mut reader = parse_jsonl(BufReader::new(data));
        assert_eq!(reader.next().unwrap().unwrap()["a"], 1);
        assert!(reader.next().unwrap().is_err());
    }

    #[test]
    fn typed_iterator_deserializes_records() {
        let data = b"{\"id\":1,\"name\":\"a\"}\n\n{\"id\":2,\"name\":\"b\"}\n";
        let rows: Vec<Row> = parse_jsonl(BufReader::new(data.as_slice()))
            .into_typed::<Row>()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[1],
            Row {
                id: 2,
                name: "b".to_string()
            }
        );
    }

    #[test]
    fn typed_iterator_reports_line_for_type_mismatch() {
        let data = b"{\"id\":1,\"name\":\"a\"}\n{\"id\":\"nope\",\"name\":\"b\"}\n";
        let mut reader = parse_jsonl(BufReader::new(data.as_slice())).into_typed::<Row>();
        reader.next().unwrap().unwrap();
        let err = reader.next().unwrap().unwrap_err();
        assert_eq!(err.line, 2);
        assert_eq!(reader.line_number(), 2);
        assert!(matches!(err.kind, JsonlErrorKind::Json(_)));
    }

    #[test]
    fn typed_iterator_propagates_parse_errors() {
        let data = b"NOT_JSON\n{\"id\":1,\"name\":\"a\"}\n";
        let mut reader = parse_jsonl(BufReader::new(data.as_slice())).into_typed::<Row>();
        let err = reader.next().unwrap().unwrap_err();
        assert_eq!(err.line, 1);
        let row = reader.next().unwrap().unwrap();
        assert_eq!(row.id, 1);
    }

    #[test]
    fn writer_round_trips_with_reader() {
        let mut buf: Vec<u8> = Vec::new();
        {
            let mut writer = JsonlWriter::new(&mut buf);
            writer.write(&serde_json::json!({"a": 1})).unwrap();
            writer.write(&serde_json::json!({"b": "two"})).unwrap();
            writer.write(&serde_json::json!([1, 2, 3])).unwrap();
            writer.flush().unwrap();
        }
        let written = String::from_utf8(buf.clone()).unwrap();
        assert_eq!(written, "{\"a\":1}\n{\"b\":\"two\"}\n[1,2,3]\n");

        // The reader should recover the exact same values.
        let back: Vec<serde_json::Value> = parse_jsonl(BufReader::new(buf.as_slice()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(back.len(), 3);
        assert_eq!(back[0]["a"], 1);
        assert_eq!(back[1]["b"], "two");
        assert_eq!(back[2], serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn failed_write_emits_nothing_and_leaves_output_valid() {
        let mut buf: Vec<u8> = Vec::new();
        {
            let mut writer = JsonlWriter::new(&mut buf);
            writer.write(&serde_json::json!({"a": 1})).unwrap();
            let err = writer.write(&FailsMidway).unwrap_err();
            assert_eq!(err.line, 2);
            assert!(matches!(err.kind, JsonlErrorKind::Json(_)));
            assert_eq!(writer.line_number(), 1);
            writer.write(&serde_json::json!({"b": 2})).unwrap();
            writer.flush().unwrap();
        }
        assert_eq!(
            String::from_utf8(buf.clone()).unwrap(),
            "{\"a\":1}\n{\"b\":2}\n"
        );
        let back: Vec<serde_json::Value> = parse_jsonl(BufReader::new(buf.as_slice()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(back.len(), 2);
    }

    #[test]
    fn write_jsonl_helper() {
        let mut buf: Vec<u8> = Vec::new();
        let values = [serde_json::json!(1), serde_json::json!(2)];
        write_jsonl(&mut buf, values.iter()).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "1\n2\n");
    }

    #[test]
    fn writer_tracks_line_numbers() {
        let mut buf: Vec<u8> = Vec::new();
        let mut writer = JsonlWriter::new(&mut buf);
        writer.write(&serde_json::json!({"ok": true})).unwrap();
        assert_eq!(writer.line_number(), 1);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn parallel_parser_collects_all_records_in_order() {
        let data = b"{\"x\":1}\n\n{\"x\":2}\n{\"x\":3}\n";
        let values = parse_jsonl_parallel(BufReader::new(data.as_slice()));
        assert_eq!(values.len(), 3);
        let values: Vec<serde_json::Value> =
            values.into_iter().collect::<Result<_, _>>().unwrap();
        assert_eq!(values[0]["x"], 1);
        assert_eq!(values[2]["x"], 3);
    }

    #[cfg(feature = "rayon")]
    #[test]
    fn parallel_parser_preserves_line_numbers_and_errors() {
        let data = b"{\"a\":1}\nNOT_JSON\n{\"c\":3}\n";
        let results = parse_jsonl_parallel(BufReader::new(data.as_slice()));
        // Line 1 OK, line 2 malformed (with its original line number), line 3 OK.
        assert!(results[0].is_ok());
        let err = results[1].as_ref().unwrap_err();
        assert_eq!(err.line, 2);
        assert!(matches!(err.kind, JsonlErrorKind::Json(_)));
        assert!(results[2].is_ok());
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn async_reader_yields_records() {
        use tokio::io::BufReader;
        let data = b"{\"a\":1}\n\n{\"b\":2}\n";
        let mut reader = crate::AsyncJsonlReader::new(BufReader::new(data.as_slice()));
        let first = reader.next().await.unwrap().unwrap();
        assert_eq!(first["a"], 1);
        let second = reader.next().await.unwrap().unwrap();
        assert_eq!(second["b"], 2);
        assert!(reader.next().await.is_none());
    }
}
