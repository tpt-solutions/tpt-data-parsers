//! Property-based round-trip tests for the JSON Lines writer/reader.

use proptest::prelude::*;
use serde_json::Value;
use std::io::Cursor;
use tpt_jsonl_stream::{parse_jsonl, write_jsonl};

fn arb_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::from),
        any::<String>().prop_map(Value::String),
    ];
    leaf.prop_recursive(3, 8, 4, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            proptest::collection::btree_map(any::<String>(), inner, 0..4)
                .prop_map(|m| Value::Object(m.into_iter().collect())),
        ]
    })
}

proptest! {
    /// `write_jsonl` -> `parse_jsonl` recovers the exact record list, including
    /// nested structures and arbitrary scalar values.
    #[test]
    fn jsonl_round_trip(values in proptest::collection::vec(arb_value(), 0..16)) {
        let mut buf = Vec::new();
        write_jsonl(&mut buf, values.iter()).unwrap();
        let back: Vec<Value> = parse_jsonl(Cursor::new(buf))
            .collect::<Result<_, _>>()
            .unwrap();
        prop_assert_eq!(back, values);
    }
}
