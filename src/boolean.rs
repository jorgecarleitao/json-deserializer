use super::error::*;

#[inline]
pub fn parse_true(values: &mut &[u8]) -> Result<(), Error> {
    let data: [u8; 4] = values
        .get(..4)
        .ok_or(Error::InvalidEOF)?
        .try_into()
        .unwrap();
    *values = &values[4..];
    if data != [b't', b'r', b'u', b'e'] {
        return Err(Error::InvalidTrueToken(data));
    };
    Ok(())
}

#[inline]
pub fn parse_false(values: &mut &[u8]) -> Result<(), Error> {
    let data: [u8; 5] = values
        .get(..5)
        .ok_or(Error::InvalidEOF)?
        .try_into()
        .unwrap();
    *values = &values[5..];
    if data != [b'f', b'a', b'l', b's', b'e'] {
        return Err(Error::InvalidFalseToken(data));
    };
    Ok(())
}
