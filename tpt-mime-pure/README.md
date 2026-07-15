# tpt-mime-pure

Pure Rust MIME type detection via magic bytes and file extension fallback. 100% `no_std` compatible.

No OS calls, no shelling out to `file`. Works in minimal Docker containers, WASM, and embedded targets.

## Features

- **Magic byte detection** — checks the file's leading bytes against known signatures
- **Extension fallback** — `detect_by_extension("pdf")` for when you only have a filename
- **`no_std` compatible** — works without the standard library (with `alloc`); disable the default `std` feature
- **No dependencies** — zero external crates
- **~20 common formats** — images, video, audio, archives, documents, binaries

## Usage

```rust
use tpt_mime_pure::{detect, MimeType};

let jpeg_header = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
assert_eq!(detect(jpeg_header), Some(MimeType::Jpeg));
println!("{}", MimeType::Jpeg.as_str()); // image/jpeg
```

### Extension fallback

```rust
use tpt_mime_pure::{detect_by_extension, MimeType};

assert_eq!(detect_by_extension("pdf"), Some(MimeType::Pdf));
assert_eq!(detect_by_extension("PDF"), Some(MimeType::Pdf)); // case-insensitive
```

### File detection (requires `std` feature, enabled by default)

```rust,ignore
use tpt_mime_pure::detect_file;

let mime = detect_file("/path/to/file.jpg").unwrap();
```

### `no_std` usage

```toml
[dependencies]
tpt-mime-pure = { version = "0.1", default-features = false }
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
| `Mp4` | `video/mp4` | ftyp at offset 4 |
| `Mkv` | `video/x-matroska` | EBML `1A 45 DF A3` |
| `WebM` | `video/webm` | EBML (same as MKV) |
| `Avi` | `video/x-msvideo` | RIFF + AVI |
| `Mp3` | `audio/mpeg` | ID3 or FF FB |
| `Wav` | `audio/wav` | RIFF + WAVE |
| `Flac` | `audio/flac` | `fLaC` |
| `Ogg` | `audio/ogg` | `OggS` |
| `Pdf` | `application/pdf` | `%PDF` |
| `Zip` | `application/zip` | `PK\x03\x04` |
| `Gzip` | `application/gzip` | `1F 8B` |
| `Tar` | `application/x-tar` | `ustar` at offset 257 |
| `Sqlite` | `application/x-sqlite3` | `SQLite format 3\0` |
| `Wasm` | `application/wasm` | `\0asm` |
| `Elf` | `application/x-elf` | `\x7FELF` |
| `PeExe` | `application/x-msdownload` | `MZ` |

## License

Licensed under either of [Apache License 2.0](../LICENSE-APACHE) or [MIT](../LICENSE-MIT) at your option.
