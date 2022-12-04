mod json_integration;

use std::borrow::Cow;

use json_deserializer::{parse, Error, Number, Object, Value};

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
        (string("c"), Value::Number(Number::Float(b"1.1", b""))),
        (string("d"), Value::Null),
        (string("e"), Value::Bool(false)),
        (string("f"), Value::Bool(true)),
        (
            string("g"),
            Value::Array(vec![
                Value::String(string("b")),
                Value::Number(Number::Integer(b"2", b"")),
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
            Value::Number(Number::Float(b"1.2", b""))
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
    let num: &str = "10";
    let array: &str = &format!("[{}]", num);

    assert_eq!(
        parse(num.as_bytes())?,
        Value::Number(Number::Integer(b"10", b""))
    );
    assert_eq!(
        parse(array.as_bytes())?,
        Value::Array(vec![Value::Number(Number::Integer(b"10", b""))])
    );
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
    let num: &str = "1E10";
    let array: &str = &format!("[{}]", num);

    assert_eq!(
        parse(num.as_bytes())?,
        Value::Number(Number::Integer(b"1", b"10"))
    );
    assert_eq!(
        parse(array.as_bytes())?,
        Value::Array(vec![Value::Number(Number::Integer(b"1", b"10"))])
    );
    Ok(())
}

#[test]
fn number_exponent1() -> Result<(), Error> {
    let num: &str = "1e-10";
    let array: &str = &format!("[{}]", num);

    assert_eq!(
        parse(num.as_bytes())?,
        Value::Number(Number::Integer(b"1", b"-10"))
    );
    assert_eq!(
        parse(array.as_bytes())?,
        Value::Array(vec![Value::Number(Number::Integer(b"1", b"-10"))])
    );
    Ok(())
}

#[test]
fn number_exponent2() -> Result<(), Error> {
    let num: &str = "8.310346185542391e275";
    let array: &str = &format!("[{}]", num);

    assert_eq!(
        parse(num.as_bytes())?,
        Value::Number(Number::Float(b"8.310346185542391", b"275"))
    );
    assert_eq!(
        parse(array.as_bytes())?,
        Value::Array(vec![Value::Number(Number::Float(b"8.310346185542391", b"275"))])
    );
    Ok(())
}

#[test]
fn number_exponent3() -> Result<(), Error> {
    let num = "1.1e+10";
    let array: &str = &format!("[{}]", num);
    let obj: &str = &format!("{{\"Value\":{}}}", num);

    assert_eq!(
        parse(num.as_bytes())?,
        Value::Number(Number::Float(b"1.1", b"+10"))
    );
    assert_eq!(
        parse(array.as_bytes())?,
        Value::Array(vec![Value::Number(Number::Float(b"1.1", b"+10"))])
    );
    let mut expected = Object::new();
    expected.insert("Value".to_string(), Value::Number(Number::Float(b"1.1", b"+10")));

    assert_eq!(
        parse(obj.as_bytes())?,
        Value::Object(expected)
    );
    Ok(())
}

#[test]
fn pretty_1() -> Result<(), Error> {
    let data: &[u8] = b"[\n  null\n]";

    let item = parse(data)?;
    assert_eq!(item, Value::Array(vec![Value::Null]));
    Ok(())
}

#[test]
fn edges() {
    assert!(parse(br#""#).is_err());
    assert!(parse(br#"""#).is_err());
    assert!(parse(br#""""#).is_ok());
    assert!(parse(br#""\u"#).is_err());
    assert!(parse(br#""\u""#).is_err());
    assert!(parse(br#""\u"""#).is_err());
    assert!(parse(br#""\u1234""#).is_ok());

    assert!(parse(br#"1"#).is_ok());
    assert!(parse(br#"11"#).is_ok());
    assert!(parse(br#"1.1"#).is_ok());
    assert!(parse(br#"1.1E-6"#).is_ok());
    assert!(parse(br#"1.1e-6"#).is_ok());
    assert!(parse(br#"1.1e+6"#).is_ok());

    assert!(parse(br#"nula"#).is_err());
    assert!(parse(br#"trua"#).is_err());
    assert!(parse(br#"falsa"#).is_err());

    assert!(parse(br#"[1.1.1]"#).is_err());
    assert!(parse(br#"[1p]"#).is_err());
    assert!(parse(br#"{"a": 1p}"#).is_err());
    assert!(parse(br#"{"a"a: 1}"#).is_err());

    assert!(parse(br#""\/""#).is_ok());
    assert!(parse(br#""\f""#).is_ok());

    // \xc3\x28 is invalid utf8
    assert!(parse(&[34, 195, 40, 34]).is_err());
}

#[test]
fn err_fmt() {
    let er = parse(br#"paa"#).err().unwrap();
    assert_eq!(format!("{}", er), "InvalidToken(112)".to_string());
    assert_eq!(format!("{:?}", er), "InvalidToken(112)".to_string());
    assert_eq!(er, Error::InvalidToken(112))
}

#[test]
fn surrogates() {
    assert!(parse(br#""\uDC00""#).is_err());

    assert!(parse(br#""\uD83C""#).is_err());

    assert!(parse(br#""\uD83C\uDF95""#).is_ok());
    assert!(parse(br#""\uD83C\uFFFF""#).is_err());
    assert!(parse(br#""\uD83C\FDF95""#).is_err());
    assert!(parse(br#""\uD83C\\""#).is_ok());
    assert!(parse(br#""\uDB00""#).is_err());
    assert!(parse(br#""\uFFFF""#).is_ok());
    assert!(parse(br#""\u\FFFF""#).is_err());
    assert!(parse(br#""\uDD00""#).is_err());
}

#[test]
fn value_fmt() {
    assert_eq!(format!("{:?}", Value::Null), "Null".to_string());
}

#[test]
fn can_clone() {
    let _ = Value::Null.clone();
}

#[cfg(feature = "preserve_order")]
#[test]
fn object() -> Result<(), Error> {
    let data: &[u8] = r#"{"u64": 1, "f64": 0.1, "utf8": "foo1", "bools": true}
    "#
    .as_bytes();

    let item = parse(data)?;

    let d = [
        (string("u64"), Value::Number(Number::Integer(b"1", b""))),
        (string("f64"), Value::Number(Number::Float(b"0.1", b""))),
        (string("utf8"), Value::String(string("foo1"))),
        (string("bools"), Value::Bool(true)),
    ]
    .into_iter()
    .map(|(key, value)| (key.into_owned(), value))
    .collect::<Object>();

    assert_eq!(item, Value::Object(d));
    Ok(())
}
