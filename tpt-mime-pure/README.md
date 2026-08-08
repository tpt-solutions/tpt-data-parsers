# tpt-mime-pure

[![docs.rs](https://docs.rs/tpt-mime-pure/badge.svg)](https://docs.rs/tpt-mime-pure)
[![crates.io](https://img.shields.io/crates/v/tpt-mime-pure.svg)](https://crates.io/crates/tpt-mime-pure)

Pure Rust MIME type detection via magic bytes and file extension fallback. 100% `no_std` compatible.

No OS calls, no shelling out to `file`. Works in minimal Docker containers, WASM, and embedded targets.

## Features

- **Magic byte detection** — checks the file's leading bytes against known signatures
- **All matches** — `detect_all` reports every signature that matched, not just the first
- **Extension fallback** — `detect_by_extension("pdf")` for when you only have a filename
- **`no_std` compatible** — works without the standard library; disable the default `std` feature
- **No dependencies** — zero external crates
- **~37 common formats** — images, video, audio, archives, documents, binaries

## Usage

```rust
use tpt_mime_pure::{detect, MimeType};

let jpeg_header = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
assert_eq!(detect(jpeg_header), Some(MimeType::Jpeg));
println!("{}", MimeType::Jpeg.as_str()); // image/jpeg
```

### All matching types

Some input matches more than one signature — a `.docx` is also a valid ZIP archive.
`detect_all` returns an allocation-free iterator over every match, most specific
first, so its first item is always what `detect` would return.

```rust
use tpt_mime_pure::{detect_all, MimeType};

let all: Vec<MimeType> = detect_all(&[0xFF, 0xD8, 0xFF, 0xE0]).collect();
assert_eq!(all, vec![MimeType::Jpeg]);

// Plain UTF-8 text with no magic bytes is recognised as `text/plain`.
assert_eq!(detect_all(b"hello world").next(), Some(MimeType::Text));
```

### Extension fallback

```rust
use tpt_mime_pure::{detect_by_extension, MimeType};

assert_eq!(detect_by_extension("pdf"), Some(MimeType::Pdf));
assert_eq!(detect_by_extension("PDF"), Some(MimeType::Pdf)); // case-insensitive
```

### File detection (requires `std` feature, enabled by default)

`detect_file` buffers up to 8 KB from disk, looping until the buffer is full or
the file ends, so signatures stored at an offset survive short reads.

```rust,ignore
use tpt_mime_pure::detect_file;

let mime = detect_file("/path/to/file.jpg").unwrap();
```

### `no_std` usage

```toml
[dependencies]
tpt-mime-pure = { version = "0.2", default-features = false }
```

## Supported Types

| Variant | MIME | Magic bytes |
|---------|------|-------------|
| `Jpeg` | `image/jpeg` | `FF D8 FF` |
| `Png` | `image/png` | `89 50 4E 47 ...` |
| `Gif` | `image/gif` | `47 49 46 38` |
| `WebP` | `image/webp` | RIFF + WEBP |
| `Bmp` | `image/bmp` | `42 4D` |
| `Ico` | `image/x-icon` | `00 00 01 00` |
| `Tiff` | `image/tiff` | II or MM header |
| `Mp4` | `video/mp4` | `ftyp` box at offset 4 (unknown brand) |
| `QuickTime` | `video/quicktime` | `ftyp` brand `qt  ` |
| `ThreeGp` | `video/3gpp` | `ftyp` brand `3gp4`/`3gp5`/`3gp6` |
| `Mkv` | `video/x-matroska` | EBML `1A 45 DF A3`, DocType `matroska` |
| `WebM` | `video/webm` | EBML `1A 45 DF A3`, DocType `webm` |
| `Avi` | `video/x-msvideo` | RIFF + AVI |
| `Heic` | `image/heic` | `ftyp` brand `heic`/`heix`/`hevc`/`hevx` |
| `Heif` | `image/heif` | `ftyp` brand `mif1` |
| `Avif` | `image/avif` | `ftyp` brand `avif`/`avis` |
| `Mp3` | `audio/mpeg` | ID3 or FF FB |
| `Wav` | `audio/wav` | RIFF + WAVE |
| `Flac` | `audio/flac` | `fLaC` |
| `Ogg` | `audio/ogg` | `OggS` |
| `Pdf` | `application/pdf` | `%PDF` |
| `Zip` | `application/zip` | `PK\x03\x04` |
| `Docx` | `application/vnd.openxmlformats-officedocument.wordprocessingml.document` | ZIP entry `word/…` |
| `Xlsx` | `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` | ZIP entry `xl/…` |
| `Pptx` | `application/vnd.openxmlformats-officedocument.presentationml.presentation` | ZIP entry `ppt/…` |
| `Jar` | `application/java-archive` | ZIP entry `META-INF/MANIFEST.MF` |
| `Gzip` | `application/gzip` | `1F 8B` |
| `Tar` | `application/x-tar` | `ustar` at offset 257 |
| `Sqlite` | `application/x-sqlite3` | `SQLite format 3\0` |
| `Wasm` | `application/wasm` | `\0asm` |
| `Elf` | `application/x-elf` | `\x7FELF` |
| `PeExe` | `application/x-msdownload` | `MZ` |
| `Zstd` | `application/zstd` | `28 B5 2F FD` |
| `Xz` | `application/x-xz` | `FD 37 7A 58 5A 00` |
| `Woff` | `font/woff` | `wOFF` |
| `Woff2` | `font/woff2` | `wOF2` |
| `JavaClass` | `application/x-java-class` | `CA FE BA BE` |

The OOXML and JAR subtypes are recognised by scanning the ZIP local file header
entry names within the first 8 KB; anything else beginning with `PK\x03\x04`
stays `Zip`.

## False positives

Detection is signature based, and several formats are identified by very short
signatures — `BM` (BMP), `MZ` (PE/EXE), `1F 8B` (gzip), `00 00 01 00` (ICO) and
`CA FE BA BE` (Java class). Byte sequences that short occur naturally in
ordinary text and in arbitrary binary data, so a match is a hint rather than
proof: a plain text file beginning with `BM` is reported as `MimeType::Bmp`, and
one beginning with `MZ` as `MimeType::PeExe`.

For untrusted input, treat the result as advisory: cross-check it against the
file extension or a real parse of the format before acting on it, and use
`detect_all` to see whether the input matched more than one signature.

## Why another MIME detector?

`mime_guess` only does extension-based guessing, and `infer` / `tree_magic` pull in
larger dependency trees. `tpt-mime-pure` is zero-dependency, `no_std`-capable, and
detects from **magic bytes** — so it works wherever the `file` binary isn't
available (minimal containers, WASM, embedded).

## License

Licensed under either of [Apache License 2.0](../LICENSE-APACHE) or [MIT](../LICENSE-MIT) at your option.
