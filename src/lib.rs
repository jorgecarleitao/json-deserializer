mod error;
mod parser;
pub use error::*;
use parser::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum State<'a> {
    /// A `null`
    Null,
    /// A string (i.e. something quoted)
    String(&'a [u8]),
    /// A Number (i.e. something starting with a number with an optional period)
    Number(&'a [u8]),
    /// A boolean (i.e. `false` or `true`)
    Boolean(bool),
    /// An object (i.e. items inside curly brackets `{}` separated by colon `:` and comma `,`)
    Object(Vec<(&'a [u8], State<'a>)>),
    /// An object (i.e. items inside squared brackets `[]` separated by comma `,`)
    Array(Vec<State<'a>>),
}

/// Parses
pub fn parse(mut json: &[u8]) -> Result<State, Error> {
    let mut mode = next_mode(json[0], &Mode::None)?;
    inner_parse(&mut json, &mut mode)
}

fn inner_parse<'b, 'a>(values: &'b mut &'a [u8], mode: &mut Mode) -> Result<State<'a>, Error> {
    next_token(values, mode)?;
    match mode {
        Mode::ObjectStart => {
            *mode = Mode::None;
            parse_object(values, mode).map(State::Object)
        }
        Mode::ArrayStart => parse_array(values, mode).map(State::Array),
        Mode::String => parse_string(values, mode).map(State::String),
        Mode::Number(false) => parse_number(values, mode).map(State::Number),
        Mode::Null(0) => {
            parse_null(values, mode)?;
            Ok(State::Null)
        }
        Mode::Boolean(true, 0) => {
            parse_true(values, mode)?;
            Ok(State::Boolean(true))
        }
        Mode::Boolean(false, 0) => {
            parse_false(values, mode)?;
            Ok(State::Boolean(false))
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
        next_token(values, mode)?;
        if *mode == Mode::ItemEnd {
            *mode = Mode::None;
            advance(values, mode)?;
        }
        if *mode == Mode::ObjectEnd {
            *mode = Mode::None;
            advance(values, mode)?;
            break;
        }
        items.push(parse_item(values, mode)?);
    }
    Ok(items)
}

fn parse_array<'b, 'a>(values: &'b mut &'a [u8], mode: &mut Mode) -> Result<Vec<State<'a>>, Error> {
    advance(values, mode)?;
    let mut items = vec![];
    loop {
        if *mode == Mode::ArrayEnd {
            *mode = Mode::None;
            advance(values, mode)?;
            break;
        }
        items.push(inner_parse(values, mode)?);
        if *mode == Mode::ItemEnd {
            *mode = Mode::None;
            advance(values, mode)?;
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
    let value = inner_parse(values, mode)?;
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
    if *mode != Mode::Number(false) {
        panic!("Expected Number, found {:?}", mode);
    }
    let string = *values;
    let mut size = 0;
    loop {
        size += 1;
        advance(values, mode)?;
        if !mode.is_number() {
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
