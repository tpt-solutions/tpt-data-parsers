//! Example: detect a file type from magic bytes and from an extension.
//!
//! Run with: `cargo run -p tpt-mime-pure --example detect`

use tpt_mime_pure::{detect, detect_by_extension, MimeType};

fn main() {
    let jpeg = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
    match detect(jpeg) {
        Some(m) => println!("magic bytes -> {} (ext: {})", m.as_str(), m.extension()),
        None => println!("magic bytes -> unknown"),
    }

    for ext in ["jpg", "heic", "webm", "mov", "xyz"] {
        match detect_by_extension(ext) {
            Some(m) => println!(".{:<5} -> {}", ext, m.as_str()),
            None => println!(".{:<5} -> unknown", ext),
        }
    }

    let _ = MimeType::Jpeg;
}
