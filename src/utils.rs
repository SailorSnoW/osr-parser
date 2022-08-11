use crate::types::{Integer, Long};
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
                    return Ok(Some(String::from("")));
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

    pub fn write_string(str: &Option<&str>, buf: &mut Vec<u8>) {
        match str {
            Some(str) => {
                let str_len = str.len() as u8;
                buf.append(&mut 0x0Bu8.to_le_bytes().to_vec());
                buf.append(&mut str_len.to_le_bytes().to_vec());
                buf.append(&mut str.as_bytes().to_vec());
            }
            None => buf.append(&mut 0u8.to_le_bytes().to_vec()),
        }
    }
}

pub mod lzma {
    use crate::error::Error;
    use xz2::stream::{Action, LzmaOptions, Stream};

    pub fn decompress_replay_data(compressed_data: &Vec<u8>) -> Result<Vec<u8>, Error> {
        let buffer = compressed_data.as_slice();
        let mut s = Vec::with_capacity(u32::MAX as usize);

        let mut lzma_decoder = Stream::new_lzma_decoder(u32::MAX as u64).unwrap();

        lzma_decoder
            .process_vec(buffer, &mut s, Action::Finish)
            .unwrap();
        Ok(s)
    }

    pub fn compress_replay_data(uncompressed_data: Vec<u8>) -> Result<Vec<u8>, Error> {
        let mut lzma_encoder = Stream::new_easy_encoder(6, xz2::stream::Check::Crc64)
            .map_err(|_| Error::NewLzmaEncoderError)?;
        let mut buffer = Vec::with_capacity(uncompressed_data.len());

        lzma_encoder
            .process_vec(&uncompressed_data, &mut buffer, Action::Finish)
            .unwrap();

        Ok(buffer)
    }
}

pub mod file {
    use crate::error::Error;
    use std::path::Path;

    pub fn ensure_replay_file(path: &Path) -> Result<(), Error> {
        match path.extension() {
            Some(extension) if extension == "osr" => Ok(()),
            Some(_) => Err(Error::NotAReplayFile {
                file: path.to_string_lossy().to_string(),
            }),
            None => Err(Error::NotAFile {
                path: path.to_string_lossy().to_string(),
            }),
        }
    }
}

pub fn ticks_to_datetime(t_ticks: Long) -> NaiveDateTime {
    NaiveDateTime::from_timestamp(((t_ticks / 10000000) - 62135596800).try_into().unwrap(), 0)
}

pub fn datetime_to_ticks(datetime: NaiveDateTime) -> Integer {
    let unix = datetime.timestamp();
    ((unix + 62135596800) * 10000000) as Integer
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
