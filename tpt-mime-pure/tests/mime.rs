use tpt_mime_pure::{detect, detect_all, detect_by_extension, MimeType};

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
    let sig: &[u8] = &[
        0x1A, 0x45, 0xDF, 0xA3, 0x8B, 0x42, 0x82, 0x88, b'm', b'a', b't', b'r', b'o', b's', b'k',
        b'a',
    ];
    assert_eq!(detect(sig), Some(MimeType::Mkv));
}

#[test]
fn integration_ebml_without_doctype_is_none() {
    assert_eq!(detect(&[0x1A, 0x45, 0xDF, 0xA3, 0x01]), None);
}

#[test]
fn integration_detect_all_matches_detect_first() {
    let inputs: [&[u8]; 3] = [&[0xFF, 0xD8, 0xFF, 0xE0], b"%PDF-1.7", b"not a real file"];
    for input in inputs {
        let all: Vec<MimeType> = detect_all(input).collect();
        assert_eq!(all.first().copied(), detect(input));
    }
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
