//! Composed example: detect a file's type with `tpt-mime-pure` and then
//! dispatch to the appropriate parser from the workspace.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p tpt-cli --example compose -- path/to/file.jsonl
//! ```
//!
//! This shows how the individual crates fit together: mime detection narrows
//! down the input (skipping binary/archive types), after which the structured
//! parsers are tried in order of specificity — exactly the idea behind the
//! `tpt sniff` subcommand.

use std::io::Read;

use tpt_geo_geojson::parse as parse_geojson;
use tpt_jsonl_stream::JsonlReader;
use tpt_logfmt_parse::parse_to_map;
use tpt_mime_pure::{detect_file, MimeType};

fn main() {
    let path = std::env::args().nth(1).expect("usage: compose <path>");

    let mime = detect_file(&path).ok().flatten();
    println!(
        "detected: {}",
        mime.map(|m| m.as_str()).unwrap_or("unknown")
    );

    let mut bytes = Vec::new();
    std::fs::File::open(&path)
        .and_then(|mut f| f.read_to_end(&mut bytes))
        .expect("read file");
    let text = String::from_utf8_lossy(&bytes);

    // Binary/archive types are not parsed further by this demo.
    if let Some(
        MimeType::Pdf
        | MimeType::Zip
        | MimeType::Gzip
        | MimeType::JavaClass
        | MimeType::Elf
        | MimeType::PeExe
        | MimeType::Wasm,
    ) = mime
    {
        println!("binary/archive document ({} bytes) — skipped", bytes.len());
        return;
    }

    // GeoJSON is the most specific structured format, so try it first.
    if parse_geojson(&text).is_ok() {
        println!("parsed as GeoJSON");
        return;
    }

    // Then a JSON Lines stream.
    let records: Vec<_> = JsonlReader::new(bytes.as_slice()).flatten().collect();
    if !records.is_empty() {
        println!("parsed {} JSON Lines record(s)", records.len());
        return;
    }

    // Finally, fall back to logfmt line parsing.
    if let Ok(pairs) = parse_to_map(&text) {
        for (k, v) in pairs {
            println!("{k} = {v}");
        }
    }
}
