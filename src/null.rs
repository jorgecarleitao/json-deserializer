use super::error::*;

#[inline]
pub fn parse_null(values: &mut &[u8]) -> Result<(), Error> {
    let data: [u8; 4] = values
        .get(..4)
        .ok_or(Error::InvalidEOF)?
        .try_into()
        .unwrap();
    *values = &values[4..];
    if data != [b'n', b'u', b'l', b'l'] {
        return Err(Error::InvalidNullToken(data));
    };
    Ok(())
}
