use crate::types::Long;
use chrono::NaiveDateTime;

pub mod read {
    use crate::error::Error;
    use crate::types::{Byte, Integer, Long, Short};
    use byteorder::{ByteOrder, LittleEndian};
    use std::io::Read;

    pub type ReadResult<T> = Result<T, Error>;

    pub fn read_byte<R: Read>(buf: &mut R) -> ReadResult<Byte> {
        let mut x = [0];
        buf.read(&mut x).map_err(|_| Error::ReadBufferingError)?;
        Ok(x[0])
    }

    pub fn read_short<R: Read>(buf: &mut R) -> ReadResult<Short> {
        let mut x = [0, 0];
        buf.read(&mut x).map_err(|_| Error::ReadBufferingError)?;
        Ok(LittleEndian::read_u16(&x))
    }

    pub fn read_integer<R: Read>(buf: &mut R) -> ReadResult<Integer> {
        let mut x = [0, 0, 0, 0];
        buf.read(&mut x).map_err(|_| Error::ReadBufferingError)?;
        Ok(LittleEndian::read_u32(&x))
    }

    pub fn read_long<R: Read>(buf: &mut R) -> ReadResult<Long> {
        let mut x = [0, 0, 0, 0, 0, 0, 0, 0];
        buf.read(&mut x).map_err(|_| Error::ReadBufferingError)?;
        Ok(LittleEndian::read_u64(&x))
    }

    pub fn read_string<R: Read>(buf: &mut R) -> ReadResult<Option<String>> {
        match read_byte(buf)? {
            0x0b => {
                let string_size = read_byte(buf)?;
                if string_size == 0 {
                    return Ok(None);
                }
                let mut x = vec![0u8; string_size as usize];
                buf.read_exact(&mut x)
                    .map_err(|_| Error::ReadBufferingError)?;
                Ok(Some(
                    String::from_utf8(x).map_err(|_| Error::CantReadString)?,
                ))
            }
            0x00 => {
                return Ok(None);
            }
            _ => return Err(Error::UnexpectedStringValue),
        }
    }
}

pub fn ticks_to_datetime(t_ticks: Long) -> NaiveDateTime {
    NaiveDateTime::from_timestamp(((t_ticks / 10000000) - 62135596800).try_into().unwrap(), 0)
}

#[cfg(test)]
#[test]
fn timestamp_to_datetime() {
    let timestamp_in_ticks = 637691351690000000;
    let datetime = ticks_to_datetime(timestamp_in_ticks);

    assert_eq!(
        datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
        "2021-10-06 16:39:29"
    )
}
