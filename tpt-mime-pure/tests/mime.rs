use tpt_mime_pure::{detect, detect_by_extension, MimeType};

#[test]
fn integration_sqlite_magic() {
    let sig: &[u8] = &[
        0x53, 0x51, 0x4C, 0x69, 0x74, 0x65, 0x20, 0x66, 0x6F, 0x72, 0x6D, 0x61, 0x74, 0x20, 0x33,
        0x00, 0x10, 0x00,
    ];
    assert_eq!(detect(sig), Some(MimeType::Sqlite));
}

#[test]
fn integration_mkv_magic() {
    assert_eq!(detect(&[0x1A, 0x45, 0xDF, 0xA3, 0x01]), Some(MimeType::Mkv));
}

#[test]
fn integration_mp4_ftyp() {
    let mut bytes = [0u8; 12];
    bytes[4..8].copy_from_slice(&[0x66, 0x74, 0x79, 0x70]);
    assert_eq!(detect(&bytes), Some(MimeType::Mp4));
}

#[test]
fn integration_extension_roundtrip() {
    let types = [
        MimeType::Jpeg,
        MimeType::Png,
        MimeType::Gif,
        MimeType::Pdf,
        MimeType::Wasm,
        MimeType::Gzip,
        MimeType::Flac,
        MimeType::Ogg,
    ];
    for t in types {
        assert_eq!(
            detect_by_extension(t.extension()),
            Some(t),
            "extension roundtrip failed for {:?}",
            t
        );
    }
}

#[test]
fn integration_all_mime_strings_nonempty() {
    let types = [
        MimeType::Jpeg,
        MimeType::Png,
        MimeType::Gif,
        MimeType::WebP,
        MimeType::Bmp,
        MimeType::Ico,
        MimeType::Tiff,
        MimeType::Mp4,
        MimeType::Mkv,
        MimeType::WebM,
        MimeType::Avi,
        MimeType::Mp3,
        MimeType::Wav,
        MimeType::Flac,
        MimeType::Ogg,
        MimeType::Pdf,
        MimeType::Zip,
        MimeType::Gzip,
        MimeType::Tar,
        MimeType::Sqlite,
        MimeType::Wasm,
        MimeType::Elf,
        MimeType::PeExe,
    ];
    for t in types {
        assert!(!t.as_str().is_empty());
        assert!(!t.extension().is_empty());
    }
}
