#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

//! Pure Rust MIME type detection via magic bytes and file extension fallback.
//! See [`detect`], [`detect_by_extension`], and [`MimeType`].

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
            Self::Gzip => "application/gzip",
            Self::Tar => "application/x-tar",
            Self::Sqlite => "application/x-sqlite3",
            Self::Wasm => "application/wasm",
            Self::Elf => "application/x-elf",
            Self::PeExe => "application/x-msdownload",
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
            Self::Gzip => "gz",
            Self::Tar => "tar",
            Self::Sqlite => "db",
            Self::Wasm => "wasm",
            Self::Elf => "elf",
            Self::PeExe => "exe",
        }
    }
}

/// Detect MIME type from the leading bytes of a file.
///
/// Checks up to the first 512 bytes against known magic byte signatures.
/// Returns `None` if no signature matches.
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

    // JPEG: FF D8 FF
    if starts_with!([0xFF, 0xD8, 0xFF]) {
        return Some(MimeType::Jpeg);
    }
    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if starts_with!([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(MimeType::Png);
    }
    // GIF: 47 49 46 38
    if starts_with!([0x47, 0x49, 0x46, 0x38]) {
        return Some(MimeType::Gif);
    }
    // WebP: "RIFF" at 0, "WEBP" at offset 8
    if starts_with!([0x52, 0x49, 0x46, 0x46]) && at_offset!(8, [0x57, 0x45, 0x42, 0x50]) {
        return Some(MimeType::WebP);
    }
    // PDF: %PDF
    if starts_with!([0x25, 0x50, 0x44, 0x46]) {
        return Some(MimeType::Pdf);
    }
    // WASM: \0asm
    if starts_with!([0x00, 0x61, 0x73, 0x6D]) {
        return Some(MimeType::Wasm);
    }
    // ELF: \x7FELF
    if starts_with!([0x7F, 0x45, 0x4C, 0x46]) {
        return Some(MimeType::Elf);
    }
    // PE/EXE: MZ
    if starts_with!([0x4D, 0x5A]) {
        return Some(MimeType::PeExe);
    }
    // GZIP: 1F 8B
    if starts_with!([0x1F, 0x8B]) {
        return Some(MimeType::Gzip);
    }
    // ZIP (also DOCX/XLSX): PK\x03\x04
    if starts_with!([0x50, 0x4B, 0x03, 0x04]) {
        return Some(MimeType::Zip);
    }
    // SQLite: "SQLite format 3\0"
    if starts_with!([
        0x53, 0x51, 0x4C, 0x69, 0x74, 0x65, 0x20, 0x66, 0x6F, 0x72, 0x6D, 0x61, 0x74, 0x20, 0x33,
        0x00
    ]) {
        return Some(MimeType::Sqlite);
    }
    // FLAC: fLaC
    if starts_with!([0x66, 0x4C, 0x61, 0x43]) {
        return Some(MimeType::Flac);
    }
    // OGG: OggS
    if starts_with!([0x4F, 0x67, 0x67, 0x53]) {
        return Some(MimeType::Ogg);
    }
    // MP3: ID3 tag or sync word FF FB/FA/F3/F2
    if starts_with!([0x49, 0x44, 0x33]) {
        return Some(MimeType::Mp3);
    }
    if len >= 2 && b[0] == 0xFF && (b[1] == 0xFB || b[1] == 0xFA || b[1] == 0xF3 || b[1] == 0xF2) {
        return Some(MimeType::Mp3);
    }
    // MKV / WebM: EBML magic 1A 45 DF A3. Both share the same EBML header but
    // declare a different DocType ("webm" vs "matroska"). Scan the leading bytes
    // for the DocType element (id 0x42 0x82) and inspect its value.
    if starts_with!([0x1A, 0x45, 0xDF, 0xA3]) {
        let limit = len.min(512);
        let mut idx = 0;
        let mut kind = MimeType::Mkv;
        while idx + 3 <= limit {
            if b[idx] == 0x42 && b[idx + 1] == 0x82 {
                let data_len = (b[idx + 2] & 0x7F) as usize;
                if idx + 3 + data_len <= limit {
                    let doc_type = &b[idx + 3..idx + 3 + data_len];
                    if doc_type == b"webm" {
                        kind = MimeType::WebM;
                        break;
                    } else if doc_type == b"matroska" {
                        kind = MimeType::Mkv;
                        break;
                    }
                }
            }
            idx += 1;
        }
        return Some(kind);
    }
    // MP4 / MOV / 3GP / HEIC / HEIF / AVIF: ISO-BMFF `ftyp` box at offset 4.
    // The 4-byte major-brand string at offset 8 distinguishes the variants;
    // unknown brands fall back to MP4.
    if at_offset!(4, [0x66, 0x74, 0x79, 0x70]) {
        if len >= 12 {
            let brand = &b[8..12];
            return Some(match brand {
                b"heic" | b"heix" | b"hevc" | b"hevx" => MimeType::Heic,
                b"mif1" => MimeType::Heif,
                b"avif" | b"avis" => MimeType::Avif,
                b"qt  " => MimeType::QuickTime,
                b"3gp4" | b"3gp5" | b"3gp6" | b"3gr6" | b"3gs6" | b"3gpp" => MimeType::ThreeGp,
                _ => MimeType::Mp4,
            });
        }
        return Some(MimeType::Mp4);
    }
    // WAV: RIFF at 0, WAVE at offset 8
    if starts_with!([0x52, 0x49, 0x46, 0x46]) && at_offset!(8, [0x57, 0x41, 0x56, 0x45]) {
        return Some(MimeType::Wav);
    }
    // AVI: RIFF at 0, AVI  at offset 8
    if starts_with!([0x52, 0x49, 0x46, 0x46]) && at_offset!(8, [0x41, 0x56, 0x49, 0x20]) {
        return Some(MimeType::Avi);
    }
    // TAR: "ustar" at offset 257
    if at_offset!(257, [0x75, 0x73, 0x74, 0x61, 0x72]) {
        return Some(MimeType::Tar);
    }
    // BMP: BM
    if starts_with!([0x42, 0x4D]) {
        return Some(MimeType::Bmp);
    }
    // ICO: 00 00 01 00
    if starts_with!([0x00, 0x00, 0x01, 0x00]) {
        return Some(MimeType::Ico);
    }
    // TIFF: II (little-endian) or MM (big-endian)
    if starts_with!([0x49, 0x49, 0x2A, 0x00]) || starts_with!([0x4D, 0x4D, 0x00, 0x2A]) {
        return Some(MimeType::Tiff);
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
    if eq("zip") || eq("docx") || eq("xlsx") || eq("pptx") || eq("jar") {
        return Some(MimeType::Zip);
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

    None
}

/// Read up to 512 bytes from `path` and detect the MIME type.
///
/// Falls back to `None` if the magic bytes are not recognised.
/// Requires the `std` feature (enabled by default).
#[cfg(feature = "std")]
pub fn detect_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Option<MimeType>> {
    use std::io::Read;
    let mut buf = [0u8; 512];
    let mut f = std::fs::File::open(path)?;
    let n = f.read(&mut buf)?;
    Ok(detect(&buf[..n]))
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
    fn ext_docx_is_zip() {
        assert_eq!(detect_by_extension("docx"), Some(MimeType::Zip));
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
}
