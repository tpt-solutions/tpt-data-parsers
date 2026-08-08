//! Property-based round-trip tests for the logfmt writer/parser.

use proptest::prelude::*;
use tpt_logfmt_parse::{parse_to_pairs, write_logfmt};

proptest! {
    /// `write_logfmt` -> `parse_to_pairs` is the identity for arbitrary keys and
    /// values, including control characters and Unicode: every special character
    /// is escaped on the way out and decoded on the way back.
    #[test]
    fn write_parse_is_identity(
        pairs in proptest::collection::vec(any::<(String, String)>(), 0..16),
    ) {
        let line = write_logfmt(pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        let back = parse_to_pairs(&line).unwrap();
        prop_assert_eq!(back, pairs);
    }

    /// Order and duplicate keys are preserved through a write/parse round trip.
    #[test]
    fn duplicates_and_order_survive(
        keys in proptest::collection::vec(any::<String>(), 1..8),
    ) {
        let pairs: Vec<(String, String)> = keys
            .iter()
            .map(|k| (k.clone(), k.clone()))
            .collect();
        let line = write_logfmt(pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        let back = parse_to_pairs(&line).unwrap();
        prop_assert_eq!(back, pairs);
    }
}
