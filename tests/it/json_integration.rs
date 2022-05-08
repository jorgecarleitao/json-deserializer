use proptest::prelude::*;
use std::collections::HashMap;

use crate::parse;
use serde_json::Value;

fn hash_to_map(values: HashMap<String, Value>) -> serde_json::Map<String, Value> {
    let mut v = serde_json::Map::<String, Value>::new();
    for (key, value) in values {
        v.insert(key, value);
    }
    v
}

fn arb_json(pretty: bool) -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<f64>().prop_map(Value::from),
        any::<i32>().prop_map(Value::from),
        ".*".prop_map(Value::String),
    ];
    leaf.prop_recursive(
        4,  // 8 levels deep
        20, // Shoot for maximum size of 256 nodes
        10, // We put up to 10 items per collection
        |inner| {
            prop_oneof![
                // Take the inner strategy and make the two recursive cases.
                prop::collection::vec(inner.clone(), 0..10).prop_map(Value::Array),
                prop::collection::hash_map(".*", inner, 0..10)
                    .prop_map(hash_to_map)
                    .prop_map(Value::Object),
            ]
        },
    )
    .prop_map(move |x| {
        let s = Value::Array(vec![x]);
        if pretty {
            serde_json::to_string_pretty(&s).unwrap()
        } else {
            serde_json::to_string(&s).unwrap()
        }
    })
}

proptest! {
    #[test]
    fn read_valid_json_doesnt_crash(a in arb_json(false)) {
        assert!(parse(a.as_bytes()).is_ok());
    }

    #[test]
    fn read_pretty_json_doesnt_crash(a in arb_json(true)) {
        assert!(parse(a.as_bytes()).is_ok());
    }
}
