#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! Pure Rust MIME type detection via magic bytes and file extension fallback.
//! See [`detect`], [`detect_all`], [`detect_by_extension`], and [`MimeType`].

/// A detected MIME type.
///
/// This enum is `#[non_exhaustive]` — new variants may be added in future releases
/// without a breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MimeType {
    // Images
    /// `image/jpeg`
    Jpeg,
    /// `image/png`
    Png,
    /// `image/gif`
    Gif,
    /// `image/webp`
    WebP,
    /// `image/bmp`
    Bmp,
    /// `image/x-icon`
    Ico,
    /// `image/tiff`
    Tiff,
    // Video
    /// `video/mp4`
    Mp4,
    /// `video/quicktime` (MOV)
    QuickTime,
    /// `video/3gpp`
    ThreeGp,
    /// `video/x-matroska`
    Mkv,
    /// `video/webm`
    WebM,
    /// `video/x-msvideo`
    Avi,
    // Images (container formats sharing the ISO-BMFF `ftyp` box)
    /// `image/heic`
    Heic,
    /// `image/heif`
    Heif,
    /// `image/avif`
    Avif,
    // Audio
    /// `audio/mpeg`
    Mp3,
    /// `audio/wav`
    Wav,
    /// `audio/flac`
    Flac,
    /// `audio/ogg`
    Ogg,
    // Documents & archives
    /// `application/pdf`
    Pdf,
    /// `application/zip`
    Zip,
    /// `application/vnd.openxmlformats-officedocument.wordprocessingml.document`
    Docx,
    /// `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet`
    Xlsx,
    /// `application/vnd.openxmlformats-officedocument.presentationml.presentation`
    Pptx,
    /// `application/java-archive`
    Jar,
    /// `application/gzip`
    Gzip,
    /// `application/x-tar`
    Tar,
    /// `application/x-sqlite3`
    Sqlite,
    // Binary / code
    /// `application/wasm`
    Wasm,
    /// `application/x-elf`
    Elf,
    /// `application/x-msdownload`
    PeExe,
    // Compressed / fonts / bytecode
    /// `application/zstd`
    Zstd,
    /// `application/x-xz`
    Xz,
    /// `font/woff`
    Woff,
    /// `font/woff2`
    Woff2,
    /// `application/x-java-class`
    JavaClass,
}

impl MimeType {
    /// The MIME type string, e.g. `"image/jpeg"`.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::Gif => "image/gif",
            Self::WebP => "image/webp",
            Self::Bmp => "image/bmp",
            Self::Ico => "image/x-icon",
            Self::Tiff => "image/tiff",
            Self::Mp4 => "video/mp4",
            Self::QuickTime => "video/quicktime",
            Self::ThreeGp => "video/3gpp",
            Self::Mkv => "video/x-matroska",
            Self::WebM => "video/webm",
            Self::Avi => "video/x-msvideo",
            Self::Heic => "image/heic",
            Self::Heif => "image/heif",
            Self::Avif => "image/avif",
            Self::Mp3 => "audio/mpeg",
            Self::Wav => "audio/wav",
            Self::Flac => "audio/flac",
            Self::Ogg => "audio/ogg",
            Self::Pdf => "application/pdf",
            Self::Zip => "application/zip",
            Self::Docx => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            Self::Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            Self::Pptx => {
                "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            }
            Self::Jar => "application/java-archive",
            Self::Gzip => "application/gzip",
            Self::Tar => "application/x-tar",
            Self::Sqlite => "application/x-sqlite3",
            Self::Wasm => "application/wasm",
            Self::Elf => "application/x-elf",
            Self::PeExe => "application/x-msdownload",
            Self::Zstd => "application/zstd",
            Self::Xz => "application/x-xz",
            Self::Woff => "font/woff",
            Self::Woff2 => "font/woff2",
            Self::JavaClass => "application/x-java-class",
        }
    }

    /// The canonical file extension (without leading dot), e.g. `"jpg"`.
    pub const fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::Gif => "gif",
            Self::WebP => "webp",
            Self::Bmp => "bmp",
            Self::Ico => "ico",
            Self::Tiff => "tiff",
            Self::Mp4 => "mp4",
            Self::QuickTime => "mov",
            Self::ThreeGp => "3gp",
            Self::Mkv => "mkv",
            Self::WebM => "webm",
            Self::Avi => "avi",
            Self::Heic => "heic",
            Self::Heif => "heif",
            Self::Avif => "avif",
            Self::Mp3 => "mp3",
            Self::Wav => "wav",
            Self::Flac => "flac",
            Self::Ogg => "ogg",
            Self::Pdf => "pdf",
            Self::Zip => "zip",
            Self::Docx => "docx",
            Self::Xlsx => "xlsx",
            Self::Pptx => "pptx",
            Self::Jar => "jar",
            Self::Gzip => "gz",
            Self::Tar => "tar",
            Self::Sqlite => "db",
            Self::Wasm => "wasm",
            Self::Elf => "elf",
            Self::PeExe => "exe",
            Self::Zstd => "zst",
            Self::Xz => "xz",
            Self::Woff => "woff",
            Self::Woff2 => "woff2",
            Self::JavaClass => "class",
        }
    }
}

/// Detect MIME type from the leading bytes of a file.
///
/// Checks the leading bytes against known magic byte signatures.
/// [`detect_file`] reads up to 8 KB from disk; every signature except the ZIP
/// entry names that identify the OOXML and JAR subtypes is found within the
/// first 512 bytes, so shorter slices work almost equally well.
/// Returns `None` if no signature matches.
///
/// When an input matches several signatures the most specific one is returned;
/// [`detect_all`] reports every match.
///
/// # False positives
///
/// Several formats are identified by very short signatures — `BM` (BMP), `MZ`
/// (PE/EXE), `1F 8B` (gzip), `00 00 01 00` (ICO) and `CA FE BA BE` (Java class).
/// Byte sequences that short occur naturally in ordinary text and in arbitrary
/// binary data, so a match is a hint rather than proof: a plain text file
/// beginning with `BM` is reported as [`MimeType::Bmp`], and one beginning with
/// `MZ` as [`MimeType::PeExe`]. For untrusted input treat the result as
/// advisory — cross-check it against the file extension or a real parse of the
/// format, and use [`detect_all`] to see whether the input was ambiguous.
///
/// # Example
///
/// ```
/// use tpt_mime_pure::{detect, MimeType};
///
/// let jpeg_header = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
/// assert_eq!(detect(jpeg_header), Some(MimeType::Jpeg));
/// ```
pub fn detect(bytes: &[u8]) -> Option<MimeType> {
    let mut sink = Sink::new(true);
    scan(bytes, &mut sink);
    sink.items[0]
}

/// Detect every MIME type whose signature matches the leading bytes.
///
/// [`detect`] returns only the first match, which hides genuinely ambiguous
/// input: a `.docx` file is also a valid ZIP archive, and a TAR archive whose
/// first member name starts with `BM` also matches the BMP signature. Matches
/// are yielded in the same priority order [`detect`] uses, so the first item is
/// always what [`detect`] would have returned, and an empty iterator means the
/// same thing as `None`.
///
/// The iterator borrows nothing, allocates nothing, and yields at most
/// [`MAX_MATCHES`] items.
///
/// # Example
///
/// ```
/// use tpt_mime_pure::{detect_all, MimeType};
///
/// let all: Vec<MimeType> = detect_all(&[0xFF, 0xD8, 0xFF, 0xE0]).collect();
/// assert_eq!(all, vec![MimeType::Jpeg]);
///
/// assert_eq!(detect_all(b"hello world").next(), None);
/// ```
pub fn detect_all(bytes: &[u8]) -> MimeMatches {
    let mut sink = Sink::new(false);
    scan(bytes, &mut sink);
    MimeMatches {
        items: sink.items,
        len: sink.len,
        pos: 0,
    }
}

/// The maximum number of matches [`detect_all`] can report for one input.
pub const MAX_MATCHES: usize = 8;

/// Iterator over the MIME types matched by [`detect_all`].
///
/// Yields at most [`MAX_MATCHES`] items, most specific first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MimeMatches {
    items: [Option<MimeType>; MAX_MATCHES],
    len: usize,
    pos: usize,
}

impl Iterator for MimeMatches {
    type Item = MimeType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.len {
            return None;
        }
        let item = self.items[self.pos];
        self.pos += 1;
        item
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len.saturating_sub(self.pos);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for MimeMatches {}

impl core::iter::FusedIterator for MimeMatches {}

struct Sink {
    items: [Option<MimeType>; MAX_MATCHES],
    len: usize,
    first_only: bool,
}

impl Sink {
    const fn new(first_only: bool) -> Self {
        Self {
            items: [None; MAX_MATCHES],
            len: 0,
            first_only,
        }
    }

    fn push(&mut self, mime: MimeType) -> bool {
        if self.len < MAX_MATCHES {
            self.items[self.len] = Some(mime);
            self.len += 1;
        }
        !self.first_only && self.len < MAX_MATCHES
    }
}

fn scan(bytes: &[u8], out: &mut Sink) {
    let b = bytes;
    let len = b.len();

    macro_rules! starts_with {
        ($sig:expr) => {
            len >= $sig.len() && b[..$sig.len()] == $sig[..]
        };
    }

    macro_rules! at_offset {
        ($offset:expr, $sig:expr) => {
            len >= $offset + $sig.len() && b[$offset..$offset + $sig.len()] == $sig[..]
        };
    }

    macro_rules! emit {
        ($mime:expr) => {
            if !out.push($mime) {
                return;
            }
        };
    }

    // JPEG: FF D8 FF
    if starts_with!([0xFF, 0xD8, 0xFF]) {
        emit!(MimeType::Jpeg);
    }
    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if starts_with!([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        emit!(MimeType::Png);
    }
    // GIF: 47 49 46 38
    if starts_with!([0x47, 0x49, 0x46, 0x38]) {
        emit!(MimeType::Gif);
    }
    // WebP: "RIFF" at 0, "WEBP" at offset 8
    if starts_with!([0x52, 0x49, 0x46, 0x46]) && at_offset!(8, [0x57, 0x45, 0x42, 0x50]) {
        emit!(MimeType::WebP);
    }
    // PDF: %PDF
    if starts_with!([0x25, 0x50, 0x44, 0x46]) {
        emit!(MimeType::Pdf);
    }
    // WASM: \0asm
    if starts_with!([0x00, 0x61, 0x73, 0x6D]) {
        emit!(MimeType::Wasm);
    }
    // ELF: \x7FELF
    if starts_with!([0x7F, 0x45, 0x4C, 0x46]) {
        emit!(MimeType::Elf);
    }
    // PE/EXE: MZ
    if starts_with!([0x4D, 0x5A]) {
        emit!(MimeType::PeExe);
    }
    // GZIP: 1F 8B
    if starts_with!([0x1F, 0x8B]) {
        emit!(MimeType::Gzip);
    }
    // ZIP (also DOCX/XLSX/PPTX/JAR): PK\x03\x04
    if starts_with!([0x50, 0x4B, 0x03, 0x04]) {
        if let Some(sub) = zip_subtype(b) {
            emit!(sub);
        }
        emit!(MimeType::Zip);
    }
    // SQLite: "SQLite format 3\0"
    if starts_with!([
        0x53, 0x51, 0x4C, 0x69, 0x74, 0x65, 0x20, 0x66, 0x6F, 0x72, 0x6D, 0x61, 0x74, 0x20, 0x33,
        0x00
    ]) {
        emit!(MimeType::Sqlite);
    }
    // FLAC: fLaC
    if starts_with!([0x66, 0x4C, 0x61, 0x43]) {
        emit!(MimeType::Flac);
    }
    // OGG: OggS
    if starts_with!([0x4F, 0x67, 0x67, 0x53]) {
        emit!(MimeType::Ogg);
    }
    // MP3: ID3 tag or sync word FF FB/FA/F3/F2
    if starts_with!([0x49, 0x44, 0x33]) {
        emit!(MimeType::Mp3);
    }
    if len >= 2 && b[0] == 0xFF && (b[1] == 0xFB || b[1] == 0xFA || b[1] == 0xF3 || b[1] == 0xF2) {
        emit!(MimeType::Mp3);
    }
    // MKV / WebM: EBML magic 1A 45 DF A3. Both share the same EBML header but
    // declare a different DocType ("webm" vs "matroska"). Walk the header's
    // child elements and inspect the DocType element (id 0x4282).
    if starts_with!([0x1A, 0x45, 0xDF, 0xA3]) {
        if let Some(kind) = ebml_doc_type(b) {
            emit!(kind);
        }
        return;
    }
    // MP4 / MOV / 3GP / HEIC / HEIF / AVIF: ISO-BMFF `ftyp` box at offset 4.
    // The 4-byte major-brand string at offset 8 distinguishes the variants;
    // unknown brands fall back to MP4.
    if at_offset!(4, [0x66, 0x74, 0x79, 0x70]) {
        let kind = if len >= 12 {
            match &b[8..12] {
                b"heic" | b"heix" | b"hevc" | b"hevx" => MimeType::Heic,
                b"mif1" => MimeType::Heif,
                b"avif" | b"avis" => MimeType::Avif,
                b"qt  " => MimeType::QuickTime,
                b"3gp4" | b"3gp5" | b"3gp6" | b"3gr6" | b"3gs6" | b"3gpp" => MimeType::ThreeGp,
                _ => MimeType::Mp4,
            }
        } else {
            MimeType::Mp4
        };
        emit!(kind);
        return;
    }
    // WAV: RIFF at 0, WAVE at offset 8
    if starts_with!([0x52, 0x49, 0x46, 0x46]) && at_offset!(8, [0x57, 0x41, 0x56, 0x45]) {
        emit!(MimeType::Wav);
    }
    // AVI: RIFF at 0, AVI  at offset 8
    if starts_with!([0x52, 0x49, 0x46, 0x46]) && at_offset!(8, [0x41, 0x56, 0x49, 0x20]) {
        emit!(MimeType::Avi);
    }
    // TAR: "ustar" at offset 257
    if at_offset!(257, [0x75, 0x73, 0x74, 0x61, 0x72]) {
        emit!(MimeType::Tar);
    }
    // BMP: BM
    if starts_with!([0x42, 0x4D]) {
        emit!(MimeType::Bmp);
    }
    // ICO: 00 00 01 00
    if starts_with!([0x00, 0x00, 0x01, 0x00]) {
        emit!(MimeType::Ico);
    }
    // TIFF: II (little-endian) or MM (big-endian)
    if starts_with!([0x49, 0x49, 0x2A, 0x00]) || starts_with!([0x4D, 0x4D, 0x00, 0x2A]) {
        emit!(MimeType::Tiff);
    }

    // Zstandard: frame magic 0xFD2FB528, little-endian on disk
    if starts_with!([0x28, 0xB5, 0x2F, 0xFD]) {
        emit!(MimeType::Zstd);
    }
    // XZ: FD 37 7A 58 5A 00
    if starts_with!([0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]) {
        emit!(MimeType::Xz);
    }
    // WOFF: "wOFF"
    if starts_with!([0x77, 0x4F, 0x46, 0x46]) {
        emit!(MimeType::Woff);
    }
    // WOFF2: "wOF2"
    if starts_with!([0x77, 0x4F, 0x46, 0x32]) {
        emit!(MimeType::Woff2);
    }
    // Java class file: CA FE BA BE
    if starts_with!([0xCA, 0xFE, 0xBA, 0xBE]) {
        emit!(MimeType::JavaClass);
    }
}

const EBML_SCAN_LIMIT: usize = 512;
const ZIP_SCAN_LIMIT: usize = 8192;
const EBML_DOC_TYPE_ID: u64 = 0x4282;

fn clamped_end(start: usize, size: u64, limit: usize) -> usize {
    usize::try_from(size)
        .ok()
        .and_then(|size| start.checked_add(size))
        .unwrap_or(limit)
        .min(limit)
}

fn ebml_vint(b: &[u8], pos: usize, mask_marker: bool, max_len: usize) -> Option<(u64, usize)> {
    let first = *b.get(pos)?;
    if first == 0 {
        return None;
    }
    let vint_len = first.leading_zeros() as usize + 1;
    if vint_len > max_len || pos + vint_len > b.len() {
        return None;
    }
    let mut value = if mask_marker {
        u64::from(first) & (0xFFu64 >> vint_len)
    } else {
        u64::from(first)
    };
    for &byte in &b[pos + 1..pos + vint_len] {
        value = (value << 8) | u64::from(byte);
    }
    Some((value, vint_len))
}

fn ebml_id(b: &[u8], pos: usize) -> Option<(u64, usize)> {
    ebml_vint(b, pos, false, 4)
}

fn ebml_size(b: &[u8], pos: usize) -> Option<(u64, usize)> {
    ebml_vint(b, pos, true, 8)
}

fn trim_trailing_nulls(value: &[u8]) -> &[u8] {
    let mut end = value.len();
    while end > 0 && value[end - 1] == 0 {
        end -= 1;
    }
    &value[..end]
}

fn ebml_doc_type(b: &[u8]) -> Option<MimeType> {
    let limit = b.len().min(EBML_SCAN_LIMIT);
    let (header_size, header_size_len) = ebml_size(b, 4)?;
    let mut pos = 4 + header_size_len;
    let header_end = clamped_end(pos, header_size, limit);
    while pos < header_end {
        let (id, id_len) = ebml_id(b, pos)?;
        let (data_size, data_size_len) = ebml_size(b, pos + id_len)?;
        let data_start = pos + id_len + data_size_len;
        let data_end = clamped_end(data_start, data_size, limit);
        if id == EBML_DOC_TYPE_ID {
            let value = trim_trailing_nulls(b.get(data_start..data_end)?);
            return match value {
                b"webm" => Some(MimeType::WebM),
                b"matroska" => Some(MimeType::Mkv),
                _ => None,
            };
        }
        pos = data_end;
    }
    None
}

fn zip_subtype(b: &[u8]) -> Option<MimeType> {
    let limit = b.len().min(ZIP_SCAN_LIMIT);
    let mut pos = 0;
    while pos + 30 <= limit {
        if b[pos] == 0x50 && b[pos + 1] == 0x4B && b[pos + 2] == 0x03 && b[pos + 3] == 0x04 {
            let name_len = u16::from_le_bytes([b[pos + 26], b[pos + 27]]) as usize;
            let name_start = pos + 30;
            let name = &b[name_start..name_start.saturating_add(name_len).min(limit)];
            if name.starts_with(b"word/") {
                return Some(MimeType::Docx);
            }
            if name.starts_with(b"xl/") {
                return Some(MimeType::Xlsx);
            }
            if name.starts_with(b"ppt/") {
                return Some(MimeType::Pptx);
            }
            if name.starts_with(b"META-INF/MANIFEST.MF") {
                return Some(MimeType::Jar);
            }
        }
        pos += 1;
    }
    None
}

/// Detect MIME type from a file extension (without leading dot, case-insensitive).
///
/// Returns `None` if the extension is not recognised.
///
/// # Example
///
/// ```
/// use tpt_mime_pure::{detect_by_extension, MimeType};
///
/// assert_eq!(detect_by_extension("jpg"), Some(MimeType::Jpeg));
/// assert_eq!(detect_by_extension("PDF"), Some(MimeType::Pdf));
/// assert_eq!(detect_by_extension("xyz"), None);
/// ```
pub fn detect_by_extension(ext: &str) -> Option<MimeType> {
    // Compare ASCII case-insensitively without allocation
    let eq = |a: &str| {
        a.len() == ext.len()
            && a.bytes()
                .zip(ext.bytes())
                .all(|(a, b)| a == b.to_ascii_lowercase())
    };

    if eq("jpg") || eq("jpeg") {
        return Some(MimeType::Jpeg);
    }
    if eq("png") {
        return Some(MimeType::Png);
    }
    if eq("gif") {
        return Some(MimeType::Gif);
    }
    if eq("webp") {
        return Some(MimeType::WebP);
    }
    if eq("bmp") {
        return Some(MimeType::Bmp);
    }
    if eq("ico") {
        return Some(MimeType::Ico);
    }
    if eq("tiff") || eq("tif") {
        return Some(MimeType::Tiff);
    }
    if eq("mp4") || eq("m4v") {
        return Some(MimeType::Mp4);
    }
    if eq("mov") || eq("qt") {
        return Some(MimeType::QuickTime);
    }
    if eq("3gp") || eq("3gpp") {
        return Some(MimeType::ThreeGp);
    }
    if eq("mkv") || eq("mk3d") {
        return Some(MimeType::Mkv);
    }
    if eq("heic") {
        return Some(MimeType::Heic);
    }
    if eq("heif") {
        return Some(MimeType::Heif);
    }
    if eq("avif") {
        return Some(MimeType::Avif);
    }
    if eq("webm") {
        return Some(MimeType::WebM);
    }
    if eq("avi") {
        return Some(MimeType::Avi);
    }
    if eq("mp3") {
        return Some(MimeType::Mp3);
    }
    if eq("wav") {
        return Some(MimeType::Wav);
    }
    if eq("flac") {
        return Some(MimeType::Flac);
    }
    if eq("ogg") || eq("oga") {
        return Some(MimeType::Ogg);
    }
    if eq("pdf") {
        return Some(MimeType::Pdf);
    }
    if eq("zip") {
        return Some(MimeType::Zip);
    }
    if eq("docx") {
        return Some(MimeType::Docx);
    }
    if eq("xlsx") {
        return Some(MimeType::Xlsx);
    }
    if eq("pptx") {
        return Some(MimeType::Pptx);
    }
    if eq("jar") {
        return Some(MimeType::Jar);
    }
    if eq("gz") || eq("gzip") {
        return Some(MimeType::Gzip);
    }
    if eq("tar") {
        return Some(MimeType::Tar);
    }
    if eq("db") || eq("sqlite") || eq("sqlite3") {
        return Some(MimeType::Sqlite);
    }
    if eq("wasm") {
        return Some(MimeType::Wasm);
    }
    if eq("elf") {
        return Some(MimeType::Elf);
    }
    if eq("exe") || eq("dll") {
        return Some(MimeType::PeExe);
    }
    if eq("zst") {
        return Some(MimeType::Zstd);
    }
    if eq("xz") {
        return Some(MimeType::Xz);
    }
    if eq("woff") {
        return Some(MimeType::Woff);
    }
    if eq("woff2") {
        return Some(MimeType::Woff2);
    }
    if eq("class") {
        return Some(MimeType::JavaClass);
    }

    None
}

/// Read up to 8 KB from `path` and detect the MIME type.
///
/// Reads repeatedly until 8 KB have been buffered or the file ends, so
/// signatures that live at an offset (`ustar` at byte 257, an EBML `DocType`)
/// are still found on readers that return short reads.
///
/// Falls back to `None` if the magic bytes are not recognised.
/// Requires the `std` feature (enabled by default).
#[cfg(feature = "std")]
pub fn detect_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Option<MimeType>> {
    use std::io::Read;
    let mut buf = [0u8; 8192];
    let mut f = std::fs::File::open(path)?;
    let mut filled = 0;
    while filled < buf.len() {
        match f.read(&mut buf[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(detect(&buf[..filled]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jpeg() {
        assert_eq!(detect(&[0xFF, 0xD8, 0xFF, 0xE0]), Some(MimeType::Jpeg));
    }

    #[test]
    fn png() {
        assert_eq!(
            detect(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
            Some(MimeType::Png)
        );
    }

    #[test]
    fn gif() {
        assert_eq!(
            detect(&[0x47, 0x49, 0x46, 0x38, 0x39, 0x61]),
            Some(MimeType::Gif)
        );
    }

    #[test]
    fn pdf() {
        assert_eq!(detect(b"%PDF-1.4"), Some(MimeType::Pdf));
    }

    #[test]
    fn wasm() {
        assert_eq!(
            detect(&[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]),
            Some(MimeType::Wasm)
        );
    }

    #[test]
    fn elf() {
        assert_eq!(detect(&[0x7F, 0x45, 0x4C, 0x46, 0x02]), Some(MimeType::Elf));
    }

    #[test]
    fn pe_exe() {
        assert_eq!(detect(&[0x4D, 0x5A, 0x90, 0x00]), Some(MimeType::PeExe));
    }

    #[test]
    fn zstd() {
        assert_eq!(
            detect(&[0x28, 0xB5, 0x2F, 0xFD, 0x00]),
            Some(MimeType::Zstd)
        );
    }

    #[test]
    fn zstd_byte_swapped_magic_is_not_zstd() {
        assert_eq!(detect(&[0x28, 0x4D, 0x18, 0x09, 0x00]), None);
    }

    #[test]
    fn xz() {
        assert_eq!(
            detect(&[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]),
            Some(MimeType::Xz)
        );
    }

    #[test]
    fn woff() {
        assert_eq!(detect(&[0x77, 0x4F, 0x46, 0x46]), Some(MimeType::Woff));
    }

    #[test]
    fn woff2() {
        assert_eq!(detect(&[0x77, 0x4F, 0x46, 0x32]), Some(MimeType::Woff2));
    }

    #[test]
    fn java_class() {
        assert_eq!(
            detect(&[0xCA, 0xFE, 0xBA, 0xBE, 0x00]),
            Some(MimeType::JavaClass)
        );
    }

    #[test]
    fn ext_compressed_font_bytecode_variants() {
        assert_eq!(detect_by_extension("zst"), Some(MimeType::Zstd));
        assert_eq!(detect_by_extension("xz"), Some(MimeType::Xz));
        assert_eq!(detect_by_extension("woff"), Some(MimeType::Woff));
        assert_eq!(detect_by_extension("woff2"), Some(MimeType::Woff2));
        assert_eq!(detect_by_extension("class"), Some(MimeType::JavaClass));
    }

    #[test]
    fn gzip() {
        assert_eq!(detect(&[0x1F, 0x8B, 0x08, 0x00]), Some(MimeType::Gzip));
    }

    #[test]
    fn zip() {
        assert_eq!(detect(&[0x50, 0x4B, 0x03, 0x04, 0x14]), Some(MimeType::Zip));
    }

    #[test]
    fn flac() {
        assert_eq!(detect(b"fLaC\x00\x00\x00\x22"), Some(MimeType::Flac));
    }

    #[test]
    fn ogg() {
        assert_eq!(detect(b"OggS\x00\x02"), Some(MimeType::Ogg));
    }

    #[test]
    fn mp3_id3() {
        assert_eq!(detect(b"ID3\x04\x00"), Some(MimeType::Mp3));
    }

    #[test]
    fn mp3_sync() {
        assert_eq!(detect(&[0xFF, 0xFB, 0x90, 0x00]), Some(MimeType::Mp3));
    }

    #[test]
    fn unknown_returns_none() {
        assert_eq!(detect(b"hello world"), None);
    }

    #[test]
    fn empty_returns_none() {
        assert_eq!(detect(&[]), None);
    }

    #[test]
    fn ext_jpg() {
        assert_eq!(detect_by_extension("jpg"), Some(MimeType::Jpeg));
        assert_eq!(detect_by_extension("jpeg"), Some(MimeType::Jpeg));
        assert_eq!(detect_by_extension("JPG"), Some(MimeType::Jpeg));
    }

    #[test]
    fn ext_pdf() {
        assert_eq!(detect_by_extension("pdf"), Some(MimeType::Pdf));
        assert_eq!(detect_by_extension("PDF"), Some(MimeType::Pdf));
    }

    #[test]
    fn ext_unknown() {
        assert_eq!(detect_by_extension("xyz"), None);
        assert_eq!(detect_by_extension(""), None);
    }

    #[test]
    fn ext_ooxml_and_jar() {
        assert_eq!(detect_by_extension("docx"), Some(MimeType::Docx));
        assert_eq!(detect_by_extension("xlsx"), Some(MimeType::Xlsx));
        assert_eq!(detect_by_extension("pptx"), Some(MimeType::Pptx));
        assert_eq!(detect_by_extension("jar"), Some(MimeType::Jar));
        assert_eq!(detect_by_extension("zip"), Some(MimeType::Zip));
    }

    #[test]
    fn mime_str_and_ext() {
        assert_eq!(MimeType::Jpeg.as_str(), "image/jpeg");
        assert_eq!(MimeType::Jpeg.extension(), "jpg");
        assert_eq!(MimeType::Wasm.as_str(), "application/wasm");
        assert_eq!(MimeType::Wasm.extension(), "wasm");
    }

    #[test]
    fn webp() {
        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(&[0x52, 0x49, 0x46, 0x46]);
        bytes[8..12].copy_from_slice(&[0x57, 0x45, 0x42, 0x50]);
        assert_eq!(detect(&bytes), Some(MimeType::WebP));
    }

    #[test]
    fn wav() {
        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(&[0x52, 0x49, 0x46, 0x46]);
        bytes[8..12].copy_from_slice(&[0x57, 0x41, 0x56, 0x45]);
        assert_eq!(detect(&bytes), Some(MimeType::Wav));
    }

    #[test]
    fn bmp() {
        assert_eq!(detect(&[0x42, 0x4D, 0x00, 0x00]), Some(MimeType::Bmp));
    }

    #[test]
    fn ico() {
        assert_eq!(detect(&[0x00, 0x00, 0x01, 0x00, 0x01]), Some(MimeType::Ico));
    }

    #[test]
    fn tiff_le() {
        assert_eq!(detect(&[0x49, 0x49, 0x2A, 0x00]), Some(MimeType::Tiff));
    }

    #[test]
    fn tiff_be() {
        assert_eq!(detect(&[0x4D, 0x4D, 0x00, 0x2A]), Some(MimeType::Tiff));
    }

    #[test]
    fn webm_via_ebml_doctype() {
        let bytes = [
            0x1A, 0x45, 0xDF, 0xA3, // EBML header id
            0x8F, // size
            0x42, 0x82, 0x84, b'w', b'e', b'b', b'm', 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(detect(&bytes), Some(MimeType::WebM));
    }

    #[test]
    fn mkv_via_ebml_doctype() {
        let bytes = [
            0x1A, 0x45, 0xDF, 0xA3, // EBML header id
            0x8F, // size
            0x42, 0x82, 0x88, b'm', b'a', b't', b'r', b'o', b's', b'k', b'a', 0, 0, 0, 0,
        ];
        assert_eq!(detect(&bytes), Some(MimeType::Mkv));
    }

    #[test]
    fn heic_via_ftyp_brand() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, b'h', b'e', b'i', b'c', 0x00, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(detect(&bytes), Some(MimeType::Heic));
    }

    #[test]
    fn heif_via_ftyp_brand() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, b'm', b'i', b'f', b'1', 0x00, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(detect(&bytes), Some(MimeType::Heif));
    }

    #[test]
    fn avif_via_ftyp_brand() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, b'a', b'v', b'i', b'f', 0x00, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(detect(&bytes), Some(MimeType::Avif));
    }

    #[test]
    fn mov_via_ftyp_brand() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, b'q', b't', b' ', b' ', 0x00, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(detect(&bytes), Some(MimeType::QuickTime));
    }

    #[test]
    fn threegp_via_ftyp_brand() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, b'3', b'g', b'p', b'4', 0x00, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(detect(&bytes), Some(MimeType::ThreeGp));
    }

    #[test]
    fn mp4_fallback_for_unknown_brand() {
        let bytes = [
            0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, b'i', b's', b'o', b'm', 0x00, 0x00,
            0x00, 0x00,
        ];
        assert_eq!(detect(&bytes), Some(MimeType::Mp4));
    }

    #[test]
    fn ext_new_variants() {
        assert_eq!(detect_by_extension("heic"), Some(MimeType::Heic));
        assert_eq!(detect_by_extension("heif"), Some(MimeType::Heif));
        assert_eq!(detect_by_extension("avif"), Some(MimeType::Avif));
        assert_eq!(detect_by_extension("mov"), Some(MimeType::QuickTime));
        assert_eq!(detect_by_extension("3gp"), Some(MimeType::ThreeGp));
    }

    #[test]
    fn ebml_without_doctype_is_none() {
        assert_eq!(detect(&[0x1A, 0x45, 0xDF, 0xA3, 0x01]), None);
    }

    #[test]
    fn ebml_unknown_doctype_is_none() {
        let bytes = [
            0x1A, 0x45, 0xDF, 0xA3, 0x8B, 0x42, 0x82, 0x88, b'w', b'e', b'i', b'r', b'd', b'!',
            b'!', b'!',
        ];
        assert_eq!(detect(&bytes), None);
    }

    #[test]
    fn ebml_doctype_past_scan_window_is_none() {
        let mut bytes = [0u8; 1024];
        bytes[..4].copy_from_slice(&[0x1A, 0x45, 0xDF, 0xA3]);
        bytes[4..8].copy_from_slice(&[0x10, 0x00, 0x02, 0xBC]);
        bytes[8] = 0xEC;
        bytes[9..13].copy_from_slice(&[0x10, 0x00, 0x02, 0x4B]);
        bytes[600..607].copy_from_slice(&[0x42, 0x82, 0x84, b'w', b'e', b'b', b'm']);
        assert_eq!(detect(&bytes), None);
    }

    #[test]
    fn matroska_via_multibyte_vint_size() {
        let bytes = [
            0x1A, 0x45, 0xDF, 0xA3, 0x8C, 0x42, 0x82, 0x40, 0x08, b'm', b'a', b't', b'r', b'o',
            b's', b'k', b'a',
        ];
        assert_eq!(detect(&bytes), Some(MimeType::Mkv));
    }

    #[test]
    fn webm_doctype_after_other_elements() {
        let bytes = [
            0x1A, 0x45, 0xDF, 0xA3, 0x8B, 0x42, 0x86, 0x81, 0x01, 0x42, 0x82, 0x84, b'w', b'e',
            b'b', b'm',
        ];
        assert_eq!(detect(&bytes), Some(MimeType::WebM));
    }

    #[test]
    fn ebml_doctype_id_inside_payload_is_skipped() {
        let bytes = [
            0x1A, 0x45, 0xDF, 0xA3, 0x8D, 0xBF, 0x84, 0x42, 0x82, 0x84, 0x77, 0x42, 0x82, 0x84,
            b'w', b'e', b'b', b'm',
        ];
        assert_eq!(detect(&bytes), Some(MimeType::WebM));
    }

    #[test]
    fn webm_doctype_with_null_padding() {
        let bytes = [
            0x1A, 0x45, 0xDF, 0xA3, 0x8B, 0x42, 0x82, 0x88, b'w', b'e', b'b', b'm', 0, 0, 0, 0,
        ];
        assert_eq!(detect(&bytes), Some(MimeType::WebM));
    }

    fn zip_with_entry(buf: &mut [u8], offset: usize, name: &[u8]) {
        buf[offset..offset + 4].copy_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
        let name_len = u16::try_from(name.len()).unwrap().to_le_bytes();
        buf[offset + 26..offset + 28].copy_from_slice(&name_len);
        buf[offset + 30..offset + 30 + name.len()].copy_from_slice(name);
    }

    #[test]
    fn docx_via_zip_entry_name() {
        let mut bytes = [0u8; 64];
        zip_with_entry(&mut bytes, 0, b"word/document.xml");
        assert_eq!(detect(&bytes), Some(MimeType::Docx));
    }

    #[test]
    fn xlsx_via_second_zip_entry_name() {
        let mut bytes = [0u8; 128];
        zip_with_entry(&mut bytes, 0, b"[Content_Types].xml");
        zip_with_entry(&mut bytes, 49, b"xl/workbook.xml");
        assert_eq!(detect(&bytes), Some(MimeType::Xlsx));
    }

    #[test]
    fn pptx_via_zip_entry_name() {
        let mut bytes = [0u8; 64];
        zip_with_entry(&mut bytes, 0, b"ppt/presentation.xml");
        assert_eq!(detect(&bytes), Some(MimeType::Pptx));
    }

    #[test]
    fn jar_via_zip_entry_name() {
        let mut bytes = [0u8; 64];
        zip_with_entry(&mut bytes, 0, b"META-INF/MANIFEST.MF");
        assert_eq!(detect(&bytes), Some(MimeType::Jar));
    }

    #[test]
    fn plain_zip_falls_back_to_zip() {
        let mut bytes = [0u8; 64];
        zip_with_entry(&mut bytes, 0, b"notes.txt");
        assert_eq!(detect(&bytes), Some(MimeType::Zip));
    }

    #[test]
    fn detect_all_single_match() {
        let mut all = detect_all(&[0xFF, 0xD8, 0xFF, 0xE0]);
        assert_eq!(all.len(), 1);
        assert_eq!(all.next(), Some(MimeType::Jpeg));
        assert_eq!(all.next(), None);
    }

    #[test]
    fn detect_all_no_match_is_empty() {
        assert_eq!(detect_all(b"hello world").count(), 0);
    }

    #[test]
    fn detect_all_reports_zip_fallback() {
        let mut bytes = [0u8; 64];
        zip_with_entry(&mut bytes, 0, b"word/document.xml");
        assert!(detect_all(&bytes).eq([MimeType::Docx, MimeType::Zip]));
    }

    #[test]
    fn detect_all_reports_overlapping_signatures() {
        let mut bytes = [0u8; 512];
        bytes[..2].copy_from_slice(b"BM");
        bytes[257..262].copy_from_slice(b"ustar");
        assert!(detect_all(&bytes).eq([MimeType::Tar, MimeType::Bmp]));
        assert_eq!(detect(&bytes), Some(MimeType::Tar));
    }

    #[test]
    fn detect_all_first_match_agrees_with_detect() {
        let inputs: [&[u8]; 5] = [
            &[0xFF, 0xD8, 0xFF, 0xE0],
            b"%PDF-1.4",
            &[0x28, 0xB5, 0x2F, 0xFD],
            &[
                0x1A, 0x45, 0xDF, 0xA3, 0x8B, 0x42, 0x82, 0x84, b'w', b'e', b'b', b'm',
            ],
            b"nothing here",
        ];
        for input in inputs {
            assert_eq!(detect_all(input).next(), detect(input));
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn detect_file_finds_offset_signature() {
        let mut path = std::env::temp_dir();
        path.push("tpt-mime-pure-detect-file-tar.bin");
        let mut data = [0u8; 2048];
        data[257..262].copy_from_slice(b"ustar");
        std::fs::write(&path, &data[..]).unwrap();
        let detected = detect_file(&path);
        let _ = std::fs::remove_file(&path);
        assert_eq!(detected.unwrap(), Some(MimeType::Tar));
    }
}
