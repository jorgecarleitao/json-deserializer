use json_parser::*;

#[test]
fn string() -> Result<(), Error> {
    let data: &[u8] = br#"" abc\" ""#;
    let mut a = data;
    let string = parse_string(&mut a, &mut Mode::String)?;
    assert_eq!(string, br#" abc\" "#);
    assert_eq!(a.len(), 0);
    Ok(())
}

#[test]
fn basics() -> Result<(), Error> {
    let data: &[u8] = br#"{
        "a": "b",
        "b":"c",
        "c": 1.1,
        "d": null,
        "e": false,
        "f": true
    }"#;
    let mut a = data;
    let mut mode = next_mode(a[0], &Mode::None)?;
    let item = parse(&mut a, &mut mode)?;
    println!("{:?}", item);
    assert!(false);
    Ok(())
}
