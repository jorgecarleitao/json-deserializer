mod error;
mod parser;
pub use error::*;
pub use parser::*;

#[derive(Debug)]
pub enum State<'a> {
    Null,
    String(&'a [u8]),
    Object(Vec<(&'a [u8], State<'a>)>),
    Number(&'a [u8]),
    Boolean(bool),
}

pub fn parse<'b, 'a>(values: &'b mut &'a [u8], mode: &mut Mode) -> Result<State<'a>, Error> {
    println!("parse: {:?}", std::str::from_utf8(values).unwrap());
    next_token(values, mode)?;
    match mode {
        Mode::ObjectStart => {
            *mode = Mode::None;
            parse_object(values, mode).map(State::Object)
        }
        Mode::String => parse_string(values, mode).map(State::String),
        Mode::Number => parse_number(values, mode).map(State::Number),
        Mode::Null(0) => {
            parse_null(values, mode)?;
            Ok(State::Null)
        }
        Mode::Boolean(true, 0) => {
            parse_true(values, mode)?;
            return Ok(State::Boolean(true));
        }
        Mode::Boolean(false, 0) => {
            parse_false(values, mode)?;
            return Ok(State::Boolean(false));
        }
        Mode::ArrayStart => {
            parse_array(values, mode)?;
            return Ok(State::Boolean(false));
        }
        _ => panic!(),
    }
}

fn parse_object<'b, 'a>(
    values: &'b mut &'a [u8],
    mode: &mut Mode,
) -> Result<Vec<(&'a [u8], State<'a>)>, Error> {
    let mut items = vec![];
    loop {
        items.push(parse_item(values, mode)?);
        if *mode == Mode::ItemEnd {
            *mode = Mode::None;
            advance(values, mode)?;
        }
        if *mode == Mode::ObjectEnd {
            break;
        }
    }
    Ok(items)
}

fn parse_array<'b, 'a>(values: &'b mut &'a [u8], mode: &mut Mode) -> Result<Vec<State<'a>>, Error> {
    advance(values, mode)?;
    let mut items = vec![];
    println!("parse_array: {:?}", std::str::from_utf8(values).unwrap());
    //todo!();
    loop {
        items.push(parse(values, mode)?);
        println!("{:?}", mode);
        if *mode == Mode::ColonSeparator {
            *mode = Mode::None;
            advance(values, mode)?;
        }
        if *mode == Mode::ArrayEnd {
            break;
        }
    }
    Ok(items)
}

fn parse_item<'b, 'a>(
    values: &'b mut &'a [u8],
    mode: &mut Mode,
) -> Result<(&'a [u8], State<'a>), Error> {
    next_token(values, mode)?;
    let key = parse_string(values, mode)?;

    next_token(values, mode)?;
    assert_eq!(*mode, Mode::ColonSeparator);
    *mode = Mode::None;
    *values = &values[1..];
    next_token(values, mode)?;
    let value = parse(values, mode)?;
    next_token(values, mode)?;
    assert!(*mode == Mode::ObjectEnd || *mode == Mode::ItemEnd);
    Ok((key, value))
}

pub fn advance(values: &mut &[u8], mode: &mut Mode) -> Result<(), Error> {
    *values = &values[1..];
    if values.is_empty() {
        *mode = Mode::Finished
    } else {
        *mode = next_mode(values[0], mode)?;
    }
    Ok(())
}

pub fn next_token(values: &mut &[u8], mode: &mut Mode) -> Result<(), Error> {
    loop {
        if values.is_empty() {
            break;
        };
        if *mode != Mode::None {
            break;
        }
        advance(values, mode)?;
    }
    Ok(())
}

pub fn parse_string<'b, 'a>(values: &'b mut &'a [u8], mode: &mut Mode) -> Result<&'a [u8], Error> {
    if !mode.is_string() {
        panic!("Expected String, found {:?}", mode);
    }
    let string = *values;
    let mut size = 0;
    loop {
        size += 1;
        advance(values, mode)?;
        if !mode.is_string() {
            break;
        }
    }
    advance(values, mode)?;
    Ok(&string[1..size])
}

pub fn parse_number<'b, 'a>(values: &'b mut &'a [u8], mode: &mut Mode) -> Result<&'a [u8], Error> {
    if *mode != Mode::Number {
        panic!("Expected Number, found {:?}", mode);
    }
    let string = *values;
    let mut size = 0;
    loop {
        size += 1;
        advance(values, mode)?;
        if *mode != Mode::Number {
            break;
        }
    }
    Ok(&string[..size])
}

pub fn parse_null(values: &mut &[u8], mode: &mut Mode) -> Result<(), Error> {
    if *mode != Mode::Null(0) {
        panic!("Expected Null, found {:?}", mode);
    }

    for _ in 0..3 {
        advance(values, mode)?;
    }
    assert_eq!(*mode, Mode::Null(3));
    advance(values, mode)?;
    Ok(())
}

pub fn parse_true(values: &mut &[u8], mode: &mut Mode) -> Result<(), Error> {
    if *mode != Mode::Boolean(true, 0) {
        panic!("Expected boolean(true, 0), found {:?}", mode);
    }

    for _ in 0..3 {
        advance(values, mode)?;
    }
    assert_eq!(*mode, Mode::Boolean(true, 3));
    advance(values, mode)?;
    Ok(())
}

pub fn parse_false(values: &mut &[u8], mode: &mut Mode) -> Result<(), Error> {
    if *mode != Mode::Boolean(false, 0) {
        panic!("Expected boolean(false, 0), found {:?}", mode);
    }

    for _ in 0..4 {
        advance(values, mode)?;
    }
    assert_eq!(*mode, Mode::Boolean(false, 4));
    advance(values, mode)?;
    Ok(())
}
