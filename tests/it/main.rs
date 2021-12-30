use json_parser::*;

#[test]
fn basics() -> Result<(), Error> {
    let data: &[u8] = br#"{
        "a": "b",
        "b": "c",
        "c": 1.1,
        "d": null,
        "e": false,
        "f": true,
        "g": ["b", 2, null, true, false, [], {}]
    }"#;

    let item = parse(data)?;
    assert_eq!(
        item,
        State::Object(vec![
            (&[b'a'], State::String(&[b'b'])),
            (&[b'b'], State::String(&[b'c'])),
            (&[b'c'], State::Number(b"1.1")),
            (&[b'd'], State::Null),
            (&[b'e'], State::Boolean(false)),
            (&[b'f'], State::Boolean(true)),
            (
                &[b'g'],
                State::Array(vec![
                    State::String(&[b'b']),
                    State::Number(&[b'2']),
                    State::Null,
                    State::Boolean(true),
                    State::Boolean(false),
                    State::Array(vec![]),
                    State::Object(vec![]),
                ])
            ),
        ])
    );
    Ok(())
}
