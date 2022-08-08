use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use crate::error::Error;
use byteorder::{ByteOrder, LittleEndian};
use types::*;

mod types {
    // Data types
    pub type Byte = u8;
    pub type Short = u16;
    pub type Integer = u32;
    pub type Long = u64;

    // Format types
    #[derive(Debug, PartialEq, Eq)]
    pub enum Gamemode {
        STD,
        TAIKO,
        CTB,
        MANIA,
    }

    impl Gamemode {
        pub fn from(byte: Byte) -> Option<Gamemode> {
            match byte {
                0x00 => Some(Self::STD),
                0x01 => Some(Self::TAIKO),
                0x02 => Some(Self::CTB),
                0x03 => Some(Self::MANIA),
                _ => None,
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Replay {
    gamemode: Gamemode,
    game_version: Integer,
    map_hash: String,
    player_name: String,
    replay_hash: String,

    number_300s: Short,
    number_100s: Short,
    number_50s: Short,
    number_gekis: Short,
    number_katus: Short,
    number_misses: Short,

    total_score: Integer,
    greatest_combo: Short,

    is_full_combo: bool,
    mods: Integer,
    life_bar_graph: Option<String>,

    timestamp: Long,
    compressed_length: Integer,
    replay_data: Vec<Byte>,
    score_id: Long,
    // TODO: additionnal mod infos
}

impl Replay {
    pub fn from_path(path: &Path) -> Result<Self, Error> {
        match path.extension() {
            Some(extension) if extension == "osr" => {
                let file = File::open(path).map_err(|_| Error::CantOpenFile)?;
                Self::from_file(&file)
            }
            Some(_) => Err(Error::NotAReplayFile {
                file: path.to_string_lossy().to_string(),
            }),
            None => Err(Error::NotAFile {
                path: path.to_string_lossy().to_string(),
            }),
        }
    }

    pub fn from_file(file: &File) -> Result<Self, Error> {
        let mut buffer = Vec::new();
        let mut reader = BufReader::new(file);

        reader
            .read_to_end(&mut buffer)
            .map_err(|_| Error::FileBufferingError)?;

        Self::from_buffer(&mut buffer.as_slice())
    }

    pub fn from_buffer<R: Read>(buf: &mut R) -> Result<Self, Error> {
        let gamemode: Gamemode =
            Gamemode::from(Self::read_byte(buf)).ok_or_else(|| Error::InvalidGamemode)?;

        let game_version = Self::read_integer(buf);

        let map_hash = Self::read_string(buf)?.unwrap();

        let player_name = Self::read_string(buf)?.unwrap();

        let replay_hash = Self::read_string(buf)?.unwrap();

        let number_300s = Self::read_short(buf);
        let number_100s = Self::read_short(buf);
        let number_50s = Self::read_short(buf);
        let number_gekis = Self::read_short(buf);
        let number_katus = Self::read_short(buf);
        let number_misses = Self::read_short(buf);

        let total_score = Self::read_integer(buf);
        let greatest_combo = Self::read_short(buf);

        let is_full_combo = match Self::read_byte(buf) {
            0x00 => false,
            0x01 => true,
            _ => return Err(Error::UnexpectedFullComboValue),
        };

        let mods = Self::read_integer(buf);

        let life_bar_graph = Self::read_string(buf)?;
        let timestamp = Self::read_long(buf);
        let compressed_length = Self::read_integer(buf);

        let mut replay_data = Vec::new();
        buf.read_to_end(&mut replay_data).unwrap();

        let score_id = 2;

        Ok(Self {
            gamemode,
            game_version,
            map_hash,
            player_name,
            replay_hash,
            number_300s,
            number_100s,
            number_50s,
            number_gekis,
            number_katus,
            number_misses,
            total_score,
            greatest_combo,
            is_full_combo,
            mods,
            life_bar_graph,
            timestamp,
            compressed_length,
            replay_data,
            score_id,
        })
    }

    fn read_byte<R: Read>(buf: &mut R) -> Byte {
        let mut x = [0];
        buf.read(&mut x).unwrap();
        x[0]
    }

    fn read_short<R: Read>(buf: &mut R) -> Short {
        let mut x = [0, 0];
        buf.read(&mut x).unwrap();
        LittleEndian::read_u16(&x)
    }

    fn read_integer<R: Read>(buf: &mut R) -> Integer {
        let mut x = [0, 0, 0, 0];
        buf.read(&mut x).unwrap();
        LittleEndian::read_u32(&x)
    }

    fn read_long<R: Read>(buf: &mut R) -> Long {
        let mut x = [0, 0, 0, 0, 0, 0, 0, 0];
        buf.read(&mut x).unwrap();
        println!("{:?}", x);
        LittleEndian::read_u64(&x)
    }

    fn read_string<R: Read>(buf: &mut R) -> Result<Option<String>, Error> {
        match Self::read_byte(buf) {
            0x0b => {
                let string_size = Self::read_byte(buf);
                let mut x = vec![0u8; string_size as usize];
                buf.read_exact(&mut x).unwrap();
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

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use super::{types::Gamemode, Replay};

    const TEST_REPLAY_FILE: &'static str = "./assets/replay-osu_1053520_3897294863.osr";

    #[test]
    fn parse_from_buffer() {
        let file = File::open(TEST_REPLAY_FILE).unwrap();

        let mut buffer = Vec::new();
        let mut reader = BufReader::new(file);

        reader.read_to_end(&mut buffer).unwrap();

        let replay = Replay::from_buffer(&mut buffer.as_slice()).unwrap();

        assert_eq!(replay.gamemode, Gamemode::STD);
        assert_eq!(replay.game_version, 20210920);
        assert_eq!(replay.map_hash, "1d74d09c4a63059835ae18b68d2b982f");
        assert_eq!(replay.player_name, "Ailes Grises");
        assert_eq!(replay.replay_hash, "56b71f87b848109cd9bed6968fadf2f2");
        assert_eq!(replay.number_300s, 364);
        assert_eq!(replay.number_100s, 10);
        assert_eq!(replay.number_50s, 0);
        assert_eq!(replay.number_gekis, 92);
        assert_eq!(replay.number_katus, 8);
        assert_eq!(replay.number_misses, 0);
        assert_eq!(replay.total_score, 7556009);
        assert_eq!(replay.greatest_combo, 519);
        assert_eq!(replay.is_full_combo, true);
        assert_eq!(replay.mods, 88);
        println!("{:?}", replay.life_bar_graph);
    }
}
