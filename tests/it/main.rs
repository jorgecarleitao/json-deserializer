mod json_integration;

use json_parser::{parse, Error, Object, StringValue, Value};

fn string(v: &str) -> StringValue {
    StringValue::Plain(v)
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
        (string("c"), Value::Number("1.1")),
        (string("d"), Value::Null),
        (string("e"), Value::Bool(false)),
        (string("f"), Value::Bool(true)),
        (
            string("g"),
            Value::Array(vec![
                Value::String(StringValue::Plain("b")),
                Value::Number("2"),
                Value::Null,
                Value::Bool(true),
                Value::Bool(false),
                Value::Array(vec![]),
                Value::Object(Default::default()),
            ]),
        ),
    ]
    .into_iter()
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
            Value::Number("1.2")
        ])
    );
    Ok(())
}

#[test]
fn empty_object() -> Result<(), Error> {
    let data: &[u8] = b"[{\"\":null}]";

    let o = [(string(""), Value::Null)].into_iter().collect::<Object>();

    let item = parse(data)?;
    assert_eq!(item, Value::Array(vec![Value::Object(o)]));
    Ok(())
}

#[test]
fn escaped() -> Result<(), Error> {
    let data: &[u8] = br#"["\n"]"#;

    let a: serde_json::Value = serde_json::from_slice(br#""\n""#).unwrap();
    if let serde_json::Value::String(a) = a {
        println!("{:?}", a.as_bytes());
    }
    println!("eol: {:?}", "\n".as_bytes());

    let item = parse(data)?;
    assert_eq!(
        item,
        Value::Array(vec![Value::String(StringValue::String("\n".into()))])
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
fn number_exponent() -> Result<(), Error> {
    let data: &[u8] = b"[1E10]";

    let item = parse(data)?;
    assert_eq!(item, Value::Array(vec![Value::Number("1E10")]));
    Ok(())
}

#[test]
fn number_exponent1() -> Result<(), Error> {
    let data: &[u8] = b"[1e10]";

    let item = parse(data)?;
    assert_eq!(item, Value::Array(vec![Value::Number("1e10")]));
    Ok(())
}

#[test]
fn pretty_1() -> Result<(), Error> {
    let data: &[u8] = b"[\n  null\n]";

    let item = parse(data)?;
    assert_eq!(item, Value::Array(vec![Value::Null]));
    Ok(())
}
