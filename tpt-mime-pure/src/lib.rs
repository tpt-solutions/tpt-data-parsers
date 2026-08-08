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
    // Archives / containers
    /// `application/x-7z-compressed`
    SevenZ,
    /// `application/vnd.rar`
    Rar,
    /// `application/x-bzip2`
    Bzip2,
    /// `application/x-ole-storage` (legacy Office, e.g. `.doc`/`.xls`/`.ppt`)
    Ole2,
    /// `application/x-mach-binary` (thin Mach-O)
    Macho,
    // Fonts
    /// `font/ttf` (TrueType)
    Ttf,
    /// `font/otf` (OpenType)
    Otf,
    // Text / markup
    /// `application/json`
    Json,
    /// `application/xml`
    Xml,
    /// `text/html`
    Html,
    /// `image/svg+xml`
    Svg,
    /// `text/plain`
    Text,
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
            Self::SevenZ => "application/x-7z-compressed",
            Self::Rar => "application/vnd.rar",
            Self::Bzip2 => "application/x-bzip2",
            Self::Ole2 => "application/x-ole-storage",
            Self::Macho => "application/x-mach-binary",
            Self::Ttf => "font/ttf",
            Self::Otf => "font/otf",
            Self::Json => "application/json",
            Self::Xml => "application/xml",
            Self::Html => "text/html",
            Self::Svg => "image/svg+xml",
            Self::Text => "text/plain",
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
            Self::SevenZ => "7z",
            Self::Rar => "rar",
            Self::Bzip2 => "bz2",
            Self::Ole2 => "doc",
            Self::Macho => "o",
            Self::Ttf => "ttf",
            Self::Otf => "otf",
            Self::Json => "json",
            Self::Xml => "xml",
            Self::Html => "html",
            Self::Svg => "svg",
            Self::Text => "txt",
        }
    }
}

/// The confidence of a MIME detection, derived from the matched signature's
/// length and how tightly it is anchored.
///
/// Short signatures that occur naturally in ordinary text or other binary
/// formats (e.g. `BM` for BMP, `MZ` for EXE, `1F 8B` for gzip) earn [`Low`],
/// because they produce frequent false positives on untrusted input. Long or
/// compound/anchored signatures (e.g. the ISO-BMFF `ftyp` brand scan, the
/// EBML `DocType` walk, a ZIP entry-name lookup, or an 8-byte PNG magic)
/// earn [`High`]. Text/markup fallbacks are always [`Low`] because any valid
/// UTF-8 buffer can be misread as `text/plain`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Confidence {
    /// Short or fallible signature — treat the result as advisory only.
    Low,
    /// Moderate, specific signature.
    Medium,
    /// Long or compound/anchored signature — high assurance.
    High,
}

impl Confidence {
    /// A stable, lowercase string name for this confidence level.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// Detect MIME type from the leading bytes of a file, also reporting the
/// [`Confidence`] (derived from the matched signature's length and anchoring).
///
/// This is [`detect`] plus a confidence score; see [`detect`] for the full
/// matching semantics and false-positive caveats.
///
/// # Example
///
/// ```
/// use tpt_mime_pure::{detect_with_confidence, MimeType, Confidence};
///
/// let jpeg_header = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
/// let (mime, conf) = detect_with_confidence(jpeg_header).unwrap();
/// assert_eq!(mime, MimeType::Jpeg);
/// assert_eq!(conf, Confidence::High);
/// ```
pub fn detect_with_confidence(bytes: &[u8]) -> Option<(MimeType, Confidence)> {
    detect(bytes).map(|m| (m, signature_confidence(m)))
}

/// Detect every MIME type whose signature matches the leading bytes, paired
/// with the [`Confidence`] of each match.
///
/// Yields the same matches (and order) as [`detect_all`], each tagged with its
/// confidence. See [`detect_all`] for the full semantics.
///
/// # Example
///
/// ```
/// use tpt_mime_pure::{detect_all_with_confidence, MimeType, Confidence};
///
/// let all = detect_all_with_confidence(&[0xFF, 0xD8, 0xFF, 0xE0])
///     .collect::<Vec<_>>();
/// assert_eq!(all, vec![(MimeType::Jpeg, Confidence::High)]);
/// ```
pub fn detect_all_with_confidence(
    bytes: &[u8],
) -> impl Iterator<Item = (MimeType, Confidence)> + '_ {
    detect_all(bytes).map(|m| (m, signature_confidence(m)))
}

/// Map a detected [`MimeType`] to the confidence of the signature that
/// produced it. Longer and more tightly-anchored signatures are more
/// trustworthy than short, generic ones.
fn signature_confidence(m: MimeType) -> Confidence {
    use Confidence::*;
    match m {
        // 2-byte offset-0 signatures: frequent false positives.
        MimeType::Bmp | MimeType::PeExe | MimeType::Gzip => Low,
        // 2-byte collision-prone magic that the docs flag as risky.
        MimeType::JavaClass => Medium,
        // 3-byte signatures: moderately specific.
        MimeType::Bzip2 => Medium,
        // 4-byte signatures (still collidable but fairly specific).
        MimeType::Ico | MimeType::Tiff => Medium,
        // Text/markup fallbacks: any valid UTF-8 buffer can match.
        MimeType::Json | MimeType::Xml | MimeType::Html | MimeType::Svg | MimeType::Text => Low,
        // Everything else is a long (>=4 byte), anchored, or compound
        // signature (ftyp brand scan, EBML DocType, ZIP entry-name, RIFF
        // subtype, ISO-BMFF, 8-byte PNG, etc.) — high assurance.
        _ => High,
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
/// assert_eq!(detect_all(b"hello world").next(), Some(MimeType::Text));
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

    // TAR: "ustar" at offset 257. Checked early so an offset-anchored signature
    // wins over the weak 2-byte offset-0 signatures below — a TAR archive whose
    // first member name begins with `MZ`, `BM`, `%PDF`, etc. is still a TAR.
    if at_offset!(257, [0x75, 0x73, 0x74, 0x61, 0x72]) {
        emit!(MimeType::Tar);
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
    // MP3: ID3 tag, or an MPEG Audio frame sync (0xFF followed by a Layer III
    // frame header for any MPEG version, including MPEG-2.5 Layer III `FF E3`).
    // The mask `(b[1] & 0xE6) == 0xE2` requires sync bits `111`, layer `01`
    // (Layer III) and version ≠ reserved (`01`).
    if starts_with!([0x49, 0x44, 0x33]) {
        emit!(MimeType::Mp3);
    }
    if len >= 2 && b[0] == 0xFF && (b[1] & 0xE6) == 0xE2 {
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
    // Both the major brand (offset 8) and the list of compatible brands are
    // consulted, so an AVIF file declared as major `mif1` but advertising `avif`
    // in its compatible-brand list is recognised as `image/avif`.
    if at_offset!(4, [0x66, 0x74, 0x79, 0x70]) {
        emit!(ftyp_kind(b));
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
    // 7z: 37 7A BC AF 27 1C
    if starts_with!([0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]) {
        emit!(MimeType::SevenZ);
    }
    // RAR: "Rar!" followed by 1A 07 (RAR4) or 1A 07 01 (RAR5)
    if starts_with!([0x52, 0x61, 0x72, 0x21, 0x1A, 0x07]) {
        emit!(MimeType::Rar);
    }
    // bzip2: "BZh"
    if starts_with!([0x42, 0x5A, 0x68]) {
        emit!(MimeType::Bzip2);
    }
    // Legacy OLE2 (older Office, e.g. .doc/.xls/.ppt): D0 CF 11 E0 A1 B1 1A E1
    if starts_with!([0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]) {
        emit!(MimeType::Ole2);
    }
    // Thin Mach-O (32/64-bit, native or byte-swapped)
    if starts_with!([0xFE, 0xED, 0xFA, 0xCE])
        || starts_with!([0xFE, 0xED, 0xFA, 0xCF])
        || starts_with!([0xCE, 0xFA, 0xED, 0xFE])
        || starts_with!([0xCF, 0xFA, 0xED, 0xFE])
    {
        emit!(MimeType::Macho);
    }
    // WOFF: "wOFF"
    if starts_with!([0x77, 0x4F, 0x46, 0x46]) {
        emit!(MimeType::Woff);
    }
    // WOFF2: "wOF2"
    if starts_with!([0x77, 0x4F, 0x46, 0x32]) {
        emit!(MimeType::Woff2);
    }
    // OpenType: "OTTO"
    if starts_with!([0x4F, 0x54, 0x54, 0x4F]) {
        emit!(MimeType::Otf);
    }
    // TrueType: 00 01 00 00 or the "true" tag
    if starts_with!([0x00, 0x01, 0x00, 0x00]) || starts_with!([0x74, 0x72, 0x75, 0x65]) {
        emit!(MimeType::Ttf);
    }
    // Java class file: CA FE BA BE
    if starts_with!([0xCA, 0xFE, 0xBA, 0xBE]) {
        emit!(MimeType::JavaClass);
    }

    // Text / markup fallback: only reached when no binary signature matched.
    if let Some(kind) = text_kind(b) {
        emit!(kind);
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

/// Pick the most specific ISO-BMFF variant by inspecting the `ftyp` major brand
/// and every compatible brand (4 bytes each, starting at offset 8). AVIF ranks
/// above HEIF so a `mif1`-major file that advertises `avif` is still AVIF.
fn ftyp_kind(b: &[u8]) -> MimeType {
    let len = b.len();
    let mut best = MimeType::Mp4;
    let mut i = 8;
    while i + 4 <= len {
        let candidate = match &b[i..i + 4] {
            b"avif" | b"avis" => MimeType::Avif,
            b"heic" | b"heix" | b"hevc" | b"hevx" => MimeType::Heic,
            b"mif1" => MimeType::Heif,
            b"qt  " => MimeType::QuickTime,
            b"3gp4" | b"3gp5" | b"3gp6" | b"3gr6" | b"3gs6" | b"3gpp" => MimeType::ThreeGp,
            _ => MimeType::Mp4,
        };
        if ftyp_rank(candidate) > ftyp_rank(best) {
            best = candidate;
        }
        i += 4;
    }
    best
}

fn ftyp_rank(m: MimeType) -> u8 {
    match m {
        MimeType::Mp4 => 0,
        MimeType::ThreeGp => 1,
        MimeType::QuickTime => 2,
        MimeType::Heif => 3,
        MimeType::Heic => 4,
        MimeType::Avif => 5,
        _ => 0,
    }
}

/// Classify a chunk of bytes that matched no binary magic signature.
///
/// Recognises the common text/markup formats (JSON, XML, HTML, SVG) and, as a
/// final fallback, any valid UTF-8 input as `text/plain`. An empty input is
/// `None` (the binary checks above would otherwise have returned for it).
fn text_kind(b: &[u8]) -> Option<MimeType> {
    if b.is_empty() {
        return None;
    }
    // Skip a UTF-8 BOM if present so the content sniff below sees real bytes.
    let content = if b.len() >= 3 && b[..3] == [0xEF, 0xBB, 0xBF] {
        &b[3..]
    } else {
        b
    };
    let trimmed = trim_ascii_whitespace(content);
    if matches!(trimmed.first(), Some(b'{' | b'[')) {
        return Some(MimeType::Json);
    }
    if trimmed.len() >= 5 && &trimmed[..5] == b"<?xml" {
        return Some(MimeType::Xml);
    }
    if trimmed.len() >= 4 && &trimmed[..4] == b"<svg" {
        return Some(MimeType::Svg);
    }
    if (trimmed.len() >= 15 && trimmed[..15].eq_ignore_ascii_case(b"<!doctype html>"))
        || (trimmed.len() >= 5 && trimmed[..5].eq_ignore_ascii_case(b"<html"))
        || (trimmed.len() >= 6 && trimmed[..6].eq_ignore_ascii_case(b"<head>"))
        || (trimmed.len() >= 6 && trimmed[..6].eq_ignore_ascii_case(b"<body>"))
    {
        return Some(MimeType::Html);
    }
    core::str::from_utf8(b)
        .ok()
        .filter(|_| is_textual(b))
        .map(|_| MimeType::Text)
}

/// Returns `true` when every byte is valid UTF-8 and the slice contains no NUL or
/// control characters other than tab/line-feed/carriage-return — i.e. it looks
/// like real text rather than an arbitrary binary buffer (which may still be
/// valid UTF-8 byte-for-byte, e.g. a run of NULs).
fn is_textual(b: &[u8]) -> bool {
    b.iter()
        .all(|&c| c == b'\t' || c == b'\n' || c == b'\r' || c >= 0x20)
}

fn trim_ascii_whitespace(mut b: &[u8]) -> &[u8] {
    while let [first, rest @ ..] = b {
        if first.is_ascii_whitespace() {
            b = rest;
        } else {
            break;
        }
    }
    while let [rest @ .., last] = b {
        if last.is_ascii_whitespace() {
            b = rest;
        } else {
            break;
        }
    }
    b
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
    if eq("7z") {
        return Some(MimeType::SevenZ);
    }
    if eq("rar") {
        return Some(MimeType::Rar);
    }
    if eq("bz2") || eq("bzip2") {
        return Some(MimeType::Bzip2);
    }
    if eq("doc") || eq("xls") || eq("ppt") {
        return Some(MimeType::Ole2);
    }
    if eq("ttf") {
        return Some(MimeType::Ttf);
    }
    if eq("otf") {
        return Some(MimeType::Otf);
    }
    if eq("json") {
        return Some(MimeType::Json);
    }
    if eq("xml") {
        return Some(MimeType::Xml);
    }
    if eq("html") || eq("htm") {
        return Some(MimeType::Html);
    }
    if eq("svg") {
        return Some(MimeType::Svg);
    }
    if eq("txt") {
        return Some(MimeType::Text);
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
        // Non-matching, invalid-UTF-8 binary-ish input stays None; a plain
        // UTF-8 string is now classified as text/plain (see text detection).
        assert_eq!(detect(&[0x01, 0x02, 0x03, 0x04]), None);
        assert_eq!(detect(b"hello world"), Some(MimeType::Text));
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
        // Non-matching binary input yields nothing; a UTF-8 string is text.
        assert_eq!(detect_all(&[0x01, 0x02, 0x03, 0x04]).count(), 0);
        assert_eq!(detect_all(b"hello world").count(), 1);
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

    // --- Phase 21b / 22 detection-quality fixes ---

    fn tar_with_first_member(name: &[u8]) -> [u8; 300] {
        let mut bytes = [0u8; 300];
        bytes[257..262].copy_from_slice(b"ustar");
        // First member name sits at offset 0 in the first 512-byte block.
        let n = name.len().min(100);
        bytes[..n].copy_from_slice(&name[..n]);
        bytes
    }

    #[test]
    fn tar_not_shadowed_by_short_offset_zero_signatures() {
        // A TAR whose first member name begins with `MZ`, `%PDF` or `BM` must
        // still be detected as a TAR, not as EXE/PDF/BMP.
        assert_eq!(
            detect(&tar_with_first_member(b"MZ-report.txt")),
            Some(MimeType::Tar)
        );
        assert_eq!(
            detect(&tar_with_first_member(b"%PDF-cover")),
            Some(MimeType::Tar)
        );
        assert_eq!(
            detect(&tar_with_first_member(b"BM-header")),
            Some(MimeType::Tar)
        );
    }

    #[test]
    fn avif_detected_from_compatible_brand() {
        // Major brand `mif1` with `avif` in the compatible-brand list.
        let bytes = [
            0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70, b'm', b'i', b'f', b'1', 0x00, 0x00,
            0x00, 0x00, b'a', b'v', b'i', b'f', 0x00, 0x00, 0x00, 0x00,
        ];
        assert_eq!(detect(&bytes), Some(MimeType::Avif));
    }

    #[test]
    fn mp3_mpeg25_layer3_sync() {
        // 0xFF 0xE3 is MPEG-2.5 Layer III — previously undetected.
        assert_eq!(detect(&[0xFF, 0xE3, 0x90, 0x00]), Some(MimeType::Mp3));
        // Reserved version (0xFF 0xE5 etc.) stays unmatched by the sync mask.
        assert_eq!(detect(&[0xFF, 0xE5, 0x00, 0x00]), None);
    }

    #[test]
    fn new_container_and_text_formats() {
        assert_eq!(
            detect(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]),
            Some(MimeType::SevenZ)
        );
        assert_eq!(
            detect(&[0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x01, 0x00]),
            Some(MimeType::Rar)
        );
        assert_eq!(detect(&[0x42, 0x5A, 0x68, 0x39]), Some(MimeType::Bzip2));
        assert_eq!(
            detect(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]),
            Some(MimeType::Ole2)
        );
        assert_eq!(detect(&[0xFE, 0xED, 0xFA, 0xCE]), Some(MimeType::Macho));
        assert_eq!(detect(&[0xCE, 0xFA, 0xED, 0xFE]), Some(MimeType::Macho));
        assert_eq!(detect(&[0x4F, 0x54, 0x54, 0x4F]), Some(MimeType::Otf));
        assert_eq!(detect(&[0x00, 0x01, 0x00, 0x00]), Some(MimeType::Ttf));
        assert_eq!(detect(b"true\x00\x01\x00"), Some(MimeType::Ttf));
    }

    #[test]
    fn text_and_markup_detection() {
        assert_eq!(detect(b"  \n  {\"a\":1}"), Some(MimeType::Json));
        assert_eq!(detect(b"[1,2,3]"), Some(MimeType::Json));
        assert_eq!(detect(b"<?xml version=\"1.0\"?>"), Some(MimeType::Xml));
        assert_eq!(
            detect(b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"),
            Some(MimeType::Svg)
        );
        assert_eq!(
            detect(b"<!DOCTYPE html><html></html>"),
            Some(MimeType::Html)
        );
        assert_eq!(detect(b"<html><body>x</body></html>"), Some(MimeType::Html));
        assert_eq!(detect(b"just some plain text"), Some(MimeType::Text));
        // UTF-8 BOM is skipped before content sniffing.
        assert_eq!(
            detect(&[0xEF, 0xBB, 0xBF, b'{', 0x22, 0x61, 0x22, 0x3A, 0x31]),
            Some(MimeType::Json)
        );
        // Invalid UTF-8 that matches no binary sig stays None.
        assert_eq!(detect(&[0x80, 0x81, 0x82, 0x83]), None);
    }

    #[test]
    fn ext_new_format_variants() {
        assert_eq!(detect_by_extension("7z"), Some(MimeType::SevenZ));
        assert_eq!(detect_by_extension("rar"), Some(MimeType::Rar));
        assert_eq!(detect_by_extension("bz2"), Some(MimeType::Bzip2));
        assert_eq!(detect_by_extension("doc"), Some(MimeType::Ole2));
        assert_eq!(detect_by_extension("ttf"), Some(MimeType::Ttf));
        assert_eq!(detect_by_extension("otf"), Some(MimeType::Otf));
        assert_eq!(detect_by_extension("json"), Some(MimeType::Json));
        assert_eq!(detect_by_extension("svg"), Some(MimeType::Svg));
        assert_eq!(detect_by_extension("html"), Some(MimeType::Html));
        assert_eq!(detect_by_extension("txt"), Some(MimeType::Text));
    }

    #[test]
    fn confidence_scores_derive_from_signature_strength() {
        // Long/anchored signatures are High.
        assert_eq!(
            detect_with_confidence(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
            Some((MimeType::Png, Confidence::High))
        );
        assert_eq!(
            detect_with_confidence(&[0x1A, 0x45, 0xDF, 0xA3, 0x8B, 0x42, 0x82, 0x84, b'w', b'e', b'b', b'm']),
            Some((MimeType::WebM, Confidence::High))
        );
        // 2-byte signatures are Low (frequent false positives).
        assert_eq!(
            detect_with_confidence(&[0x4D, 0x5A, 0x90, 0x00]),
            Some((MimeType::PeExe, Confidence::Low))
        );
        assert_eq!(
            detect_with_confidence(&[0x42, 0x4D, 0x00, 0x00]),
            Some((MimeType::Bmp, Confidence::Low))
        );
        // Text fallback is Low.
        assert_eq!(
            detect_with_confidence(b"just some plain text"),
            Some((MimeType::Text, Confidence::Low))
        );
        // 4-byte signatures that are still collidable are Medium.
        assert_eq!(
            detect_with_confidence(&[0xCA, 0xFE, 0xBA, 0xBE, 0x00]),
            Some((MimeType::JavaClass, Confidence::Medium))
        );
        assert_eq!(Confidence::Low.as_str(), "low");
        assert_eq!(Confidence::High.as_str(), "high");
        assert!(Confidence::Low < Confidence::High);
    }

    #[test]
    fn detect_all_with_confidence_agrees_with_detect_all() {
        let all: Vec<(MimeType, Confidence)> =
            detect_all_with_confidence(&[0xFF, 0xD8, 0xFF, 0xE0]).collect();
        assert_eq!(all, vec![(MimeType::Jpeg, Confidence::High)]);

        let mut bytes = [0u8; 512];
        bytes[..2].copy_from_slice(b"BM");
        bytes[257..262].copy_from_slice(b"ustar");
        let all: Vec<(MimeType, Confidence)> =
            detect_all_with_confidence(&bytes).collect();
        assert_eq!(
            all,
            vec![(MimeType::Tar, Confidence::High), (MimeType::Bmp, Confidence::Low)]
        );
    }
}
