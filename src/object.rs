use alloc::borrow::Cow;

use crate::{
    parser::{current_token, parse_value, skip_unused},
    string::parse_string,
    Object, Value,
};

use super::error::*;

// assumes that `values` contains `{`
pub fn parse_object<'b, 'a>(values: &'b mut &'a [u8]) -> Result<Object<'a>, Error> {
    *values = &values[1..];
    let mut items = Object::new();
    loop {
        skip_unused(values);
        let token = current_token(values)?;
        if token == b'}' {
            *values = &values[1..];
            break;
        };
        if !items.is_empty() {
            if token != b',' {
                return Err(Error::MissingComma(values[0]));
            }
            *values = &values[1..]; // consume ","
        }

        let (k, v) = parse_item(values)?;
        items.insert(k.into_owned(), v);
    }
    Ok(items)
}

#[inline]
fn parse_item<'b, 'a>(values: &'b mut &'a [u8]) -> Result<(Cow<'a, str>, Value<'a>), Error> {
    skip_unused(values);
    let key = parse_string(values)?;

    skip_unused(values);
    let token = current_token(values)?;
    if token != b':' {
        return Err(Error::InvalidToken(token));
    };
    *values = &values[1..];

    let value = parse_value(values)?;
    Ok((key, value))
}
