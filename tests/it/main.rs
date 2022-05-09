mod json_integration;

use std::borrow::Cow;

use json_deserializer::{parse, Error, Object, Value};

fn string(v: &str) -> Cow<str> {
    Cow::Borrowed(v)
}

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

    let d = [
        (string("a"), Value::String(string("b"))),
        (string("b"), Value::String(string("c"))),
        (string("c"), Value::Number(b"1.1")),
        (string("d"), Value::Null),
        (string("e"), Value::Bool(false)),
        (string("f"), Value::Bool(true)),
        (
            string("g"),
            Value::Array(vec![
                Value::String(string("b")),
                Value::Number(b"2"),
                Value::Null,
                Value::Bool(true),
                Value::Bool(false),
                Value::Array(vec![]),
                Value::Object(Default::default()),
            ]),
        ),
    ]
    .into_iter()
    .map(|(key, value)| (key.into_owned(), value))
    .collect::<Object>();

    assert_eq!(item, Value::Object(d));
    Ok(())
}

#[test]
fn comma_and_string() -> Result<(), Error> {
    let data: &[u8] = br#"[",", "1.2", 1.2]"#;

    let item = parse(data)?;
    assert_eq!(
        item,
        Value::Array(vec![
            Value::String(string(",")),
            Value::String(string("1.2")),
            Value::Number(b"1.2")
        ])
    );
    Ok(())
}

#[test]
fn empty_object() -> Result<(), Error> {
    let data: &[u8] = b"[{\"\":null}]";

    let o = [(string("").into_owned(), Value::Null)]
        .into_iter()
        .collect::<Object>();

    let item = parse(data)?;
    assert_eq!(item, Value::Array(vec![Value::Object(o)]));
    Ok(())
}

#[test]
fn escaped() -> Result<(), Error> {
    let data: &[u8] = br#"["\n"]"#;

    /*
    let a: serde_json::Value = serde_json::from_slice(br#""\n""#).unwrap();
    if let serde_json::Value::String(a) = a {
        println!("{:?}", a.as_bytes());
    }
    println!("eol: {:?}", "\n".as_bytes());
    */

    let item = parse(data)?;
    assert_eq!(
        item,
        Value::Array(vec![Value::String(Cow::Owned("\n".into()))])
    );
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
    assert_eq!(item, Value::Array(vec![Value::String(string(""))]));
    Ok(())
}

#[test]
fn number() -> Result<(), Error> {
    let data: &str = "[10]";

    let item = parse(data.as_bytes())?;
    assert_eq!(item, Value::Array(vec![Value::Number(b"10")]));
    Ok(())
}

#[test]
fn utf8() -> Result<(), Error> {
    let data: &str = "[\"Ç\"]";

    let item = parse(data.as_bytes())?;
    assert_eq!(item, Value::Array(vec![Value::String(string("Ç"))]));
    Ok(())
}

#[test]
fn escaped_1() -> Result<(), Error> {
    let data: &str = r#"["\n"]"#;

    let item = parse(data.as_bytes())?;
    assert_eq!(
        item,
        Value::Array(vec![Value::String(Cow::Owned("\n".to_string()))])
    );
    Ok(())
}

#[test]
fn escaped_error() -> Result<(), Error> {
    let data: &str = r#"["\"]"#;

    assert!(parse(data.as_bytes()).is_err());
    Ok(())
}

#[test]
fn codepoint() -> Result<(), Error> {
    let data: &str = r#"["\u20AC"]"#;

    let item = parse(data.as_bytes())?;
    assert_eq!(
        item,
        Value::Array(vec![Value::String(Cow::Owned("€".to_string()))])
    );
    Ok(())
}

#[test]
fn multiple_escape() -> Result<(), Error> {
    let data: &[u8] = b"[\"\\\\A\"]";

    let item = parse(data)?;
    assert_eq!(
        item,
        Value::Array(vec![Value::String(Cow::Owned("\\A".to_string()))])
    );
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
