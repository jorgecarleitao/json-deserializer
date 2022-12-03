use alloc::vec::Vec;

use crate::{
    parser::{current_token, parse_value, skip_unused},
    Value,
};

use super::error::*;

pub fn parse_array<'b, 'a>(values: &'b mut &'a [u8]) -> Result<Vec<Value<'a>>, Error> {
    *values = &values[1..];
    let mut items = vec![];
    loop {
        skip_unused(values);
        let token = current_token(values)?;
        if token == b']' {
            *values = &values[1..];
            break;
        };
        if !items.is_empty() {
            if token != b',' {
                return Err(Error::MissingComma(token));
            } else {
                *values = &values[1..]; // consume ","
            }
        }

        items.push(parse_value(values)?);
    }
    Ok(items)
}
