use std::{borrow::Cow, collections::BTreeMap};

use crate::{
    parser::{current_token, parse_value, skip_unused},
    string::parse_string,
    Object, Value,
};

use super::error::*;

// assumes that `values` contains `{`
pub fn parse_object<'b, 'a>(values: &'b mut &'a [u8]) -> Result<Object<'a>, Error> {
    *values = &values[1..];
    let mut items = BTreeMap::new();
    loop {
        skip_unused(values);
        let token = current_token(values)?;
        if token == b'}' {
            *values = &values[1..];
            break;
        };
        if !items.is_empty() {
            if token != b',' {
                return Err(Error::OutOfSpec(OutOfSpecError::MissingComa(values[0])));
            } else {
                *values = &values[1..]; // consume ","
            }
        }

        let (k, v) = parse_item(values)?;
        items.insert(k.into_owned(), v);
    }
    Ok(items)
}

fn parse_item<'b, 'a>(values: &'b mut &'a [u8]) -> Result<(Cow<'a, str>, Value<'a>), Error> {
    skip_unused(values);
    let key = parse_string(values)?;

    skip_unused(values);
    let token = current_token(values)?;
    if token != b':' {
        return Err(Error::OutOfSpec(OutOfSpecError::InvalidToken(values[0])));
    };
    *values = &values[1..];

    skip_unused(values);
    let value = parse_value(values)?;
    Ok((key, value))
}
