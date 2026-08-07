#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = tpt_mime_pure::detect(data);
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = tpt_mime_pure::detect_by_extension(s);
    }
});
