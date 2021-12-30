mod json_integration;

use json_parser::{parse, Error, Value};

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
        Value::Object(vec![
            (&[b'a'], Value::String(&[b'b'])),
            (&[b'b'], Value::String(&[b'c'])),
            (&[b'c'], Value::Number(b"1.1")),
            (&[b'd'], Value::Null),
            (&[b'e'], Value::Boolean(false)),
            (&[b'f'], Value::Boolean(true)),
            (
                &[b'g'],
                Value::Array(vec![
                    Value::String(&[b'b']),
                    Value::Number(&[b'2']),
                    Value::Null,
                    Value::Boolean(true),
                    Value::Boolean(false),
                    Value::Array(vec![]),
                    Value::Object(vec![]),
                ])
            ),
        ])
    );
    Ok(())
}

#[test]
fn comma_and_string() -> Result<(), Error> {
    let data: &[u8] = br#"[",", "1.2", 1.2]"#;

    let item = parse(data)?;
    assert_eq!(
        item,
        Value::Array(vec![
            Value::String(b","),
            Value::String(b"1.2"),
            Value::Number(b"1.2")
        ])
    );
    Ok(())
}

#[test]
fn empty_object() -> Result<(), Error> {
    let data: &[u8] = b"[{\"\":null}]";

    let item = parse(data)?;
    assert_eq!(
        item,
        Value::Array(vec![Value::Object(vec![(b"", Value::Null)])])
    );
    Ok(())
}

#[test]
fn escaped() -> Result<(), Error> {
    let data: &[u8] = br#"["\\n"]"#;

    let item = parse(data)?;
    assert_eq!(item, Value::Array(vec![Value::String(br"\\n")]));
    Ok(())
}

#[test]
fn null() -> Result<(), Error> {
    let data: &[u8] = b"[null]";

    let item = parse(data)?;
    assert_eq!(item, Value::Array(vec![Value::Null]));
    Ok(())
}

#[test]
fn empty_string() -> Result<(), Error> {
    let data: &[u8] = b"[\"\"]";

    let item = parse(data)?;
    assert_eq!(item, Value::Array(vec![Value::String(b"")]));
    Ok(())
}

#[test]
fn number_exponent() -> Result<(), Error> {
    let data: &[u8] = b"[1E10]";

    let item = parse(data)?;
    assert_eq!(item, Value::Array(vec![Value::Number(b"1E10")]));
    Ok(())
}

#[test]
fn number_exponent1() -> Result<(), Error> {
    let data: &[u8] = b"[1e10]";

    let item = parse(data)?;
    assert_eq!(item, Value::Array(vec![Value::Number(b"1e10")]));
    Ok(())
}

#[test]
fn pretty_1() -> Result<(), Error> {
    let data: &[u8] = b"[\n  null\n]";

    let item = parse(data)?;
    assert_eq!(item, Value::Array(vec![Value::Null]));
    Ok(())
}
