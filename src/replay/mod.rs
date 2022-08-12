use crate::error::Error;
use crate::types::*;
use replay_data::*;
use std::borrow::Borrow;
use std::fs::{self, File};
use std::io::{BufReader, Cursor, Read};
use std::path::Path;
use std::str::FromStr;

use crate::utils::file::ensure_replay_file;
use crate::utils::lzma::decompress_replay_data;
use crate::utils::read::*;
use crate::utils::*;
use chrono::NaiveDateTime;

mod replay_data;

/// Structure of a replay containing parsed values
#[derive(Debug, Default)]
pub struct Replay {
    /// Game mode of the replay (0 = osu! Standard, 1 = Taiko, 2 = Catch the Beat, 3 = osu!mania)
    pub gamemode: Gamemode,
    /// Version of the game when the replay was created (ex. 20131216)
    pub game_version: Integer,
    /// osu! beatmap MD5 hash
    pub map_hash: String,
    pub player_name: String,
    /// osu! replay MD5 hash (includes certain properties of the replay)
    pub replay_hash: String,

    pub number_300s: Short,
    /// Number of 100s in standard, 150s in Taiko, 100s in CTB, 100s in mania
    pub number_100s: Short,
    /// Number of 50s in standard, small fruit in CTB, 50s in mania
    pub number_50s: Short,
    /// Number of Gekis in standard, Max 300s in mania
    pub number_gekis: Short,
    /// Number of Katus in standard, 200s in mania
    pub number_katus: Short,
    pub number_misses: Short,

    /// Total score displayed on the score report
    pub total_score: Integer,
    /// Greatest combo displayed on the score report
    pub greatest_combo: Short,

    /// If the score is a Perfect/full combo
    /// (true = no misses and no slider breaks and no early finished sliders)
    pub is_full_combo: bool,
    /// Mods used (combination)
    pub mods: Mods,
    /// Life bar graph: comma separated pairs u/v.
    /// u is the time in milliseconds into the song,
    /// v is a floating point value from 0 - 1 that represents the amount of life you have at the given time
    /// (0 = life bar is empty, 1= life bar is full)
    pub life_bar_graph: Option<String>,

    /// Parsed date and time of the play from the ticks timestamp
    pub play_date: NaiveDateTime,

    /// Uncompressed and parsed replay data
    pub replay_data: ReplayData,
    /// Online score ID
    pub score_id: Long,
    // TODO: additionnal mod infos
}

impl Replay {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(path: &Path) -> Result<Self, Error> {
        ensure_replay_file(path)?;

        let file = File::open(path).map_err(|_| Error::CantOpenFile)?;
        file.borrow().try_into()
    }

    pub fn write(self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        ensure_replay_file(path)?;

        let buffer: Vec<u8> = self.try_into()?;
        Ok(fs::write(path, buffer)?)
    }

    fn read_play_date<R: Read>(buf: &mut R) -> ReadResult<NaiveDateTime> {
        let timestamp_ticks = read_long(buf)?;
        Ok(ticks_to_datetime(timestamp_ticks))
    }
}

impl TryFrom<Replay> for Vec<u8> {
    type Error = Error;

    fn try_from(replay: Replay) -> Result<Self, Error> {
        let mut buffer = Vec::<u8>::new();

        buffer.push(replay.gamemode.borrow().into());
        buffer.append(&mut replay.game_version.to_le_bytes().to_vec());
        write_string(&Some(&replay.map_hash), &mut buffer);
        write_string(&Some(&replay.player_name), &mut buffer);
        write_string(&Some(&replay.replay_hash), &mut buffer);
        buffer.append(&mut replay.number_300s.to_le_bytes().to_vec());
        buffer.append(&mut replay.number_100s.to_le_bytes().to_vec());
        buffer.append(&mut replay.number_50s.to_le_bytes().to_vec());
        buffer.append(&mut replay.number_gekis.to_le_bytes().to_vec());
        buffer.append(&mut replay.number_katus.to_le_bytes().to_vec());
        buffer.append(&mut replay.number_misses.to_le_bytes().to_vec());
        buffer.append(&mut replay.total_score.to_le_bytes().to_vec());
        buffer.append(&mut replay.greatest_combo.to_le_bytes().to_vec());
        buffer.push(replay.is_full_combo.into());
        buffer.append(&mut replay.mods.bits().to_le_bytes().to_vec());
        write_string(&replay.life_bar_graph.as_deref(), &mut buffer);
        buffer.append(&mut datetime_to_ticks(replay.play_date).to_le_bytes().to_vec());
        let mut replay_data_compressed: Vec<u8> = replay.replay_data.borrow().try_into()?;
        buffer.append(
            &mut (replay_data_compressed.len() as Integer)
                .to_le_bytes()
                .to_vec(),
        );
        buffer.append(&mut replay_data_compressed);
        buffer.append(&mut replay.score_id.to_le_bytes().to_vec());

        Ok(buffer)
    }
}

impl TryFrom<Vec<u8>> for Replay {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let buffer = &mut Cursor::new(value);

        let gamemode: Gamemode = Gamemode::try_from(read::read_byte(buffer)?)?;

        let game_version = read::read_integer(buffer)?;

        let map_hash = read::read_string(buffer)?.unwrap_or_default();
        let player_name = read::read_string(buffer)?.unwrap_or_default();
        let replay_hash = read::read_string(buffer)?.unwrap_or_default();

        let number_300s = read::read_short(buffer)?;
        let number_100s = read::read_short(buffer)?;
        let number_50s = read::read_short(buffer)?;
        let number_gekis = read::read_short(buffer)?;
        let number_katus = read::read_short(buffer)?;
        let number_misses = read::read_short(buffer)?;

        let total_score = read::read_integer(buffer)?;
        let greatest_combo = read::read_short(buffer)?;

        let is_full_combo = match read::read_byte(buffer)? {
            0x00 => false,
            0x01 => true,
            _ => return Err(Error::UnexpectedFullComboValue),
        };

        let mods = read::read_integer(buffer)?.into();
        let life_bar_graph = read::read_string(buffer)?;
        let play_date = Self::read_play_date(buffer)?;
        let compressed_length = read::read_integer(buffer)?;

        let mut compressed_replay_data = vec![0u8; compressed_length as usize];
        buffer
            .read(&mut compressed_replay_data)
            .map_err(|_| Error::ReadBufferingError)?;

        let decompressed_replay_data = decompress_replay_data(&compressed_replay_data)?;

        let replay_data =
            ReplayData::from_str(&String::from_utf8(decompressed_replay_data).unwrap_or_default())?;

        let score_id = read::read_long(buffer)?;

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
            play_date,
            replay_data,
            score_id,
        })
    }
}

impl TryFrom<&File> for Replay {
    type Error = Error;

    fn try_from(value: &File) -> Result<Self, Self::Error> {
        let mut buffer = Vec::new();
        let mut reader = BufReader::new(value);

        reader
            .read_to_end(&mut buffer)
            .map_err(|_| Error::FileBufferingError)?;

        buffer.try_into()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{Gamemode, Mods, Replay};

    const TEST_REPLAY_FILE: &'static str = "./assets/examples/replay-test.osr";
    const TEST_NEW_REPLAY_FILE: &'static str = "./assets/examples/replay-new.osr";

    #[test]
    fn open_replay() {
        let replay_path = Path::new(TEST_REPLAY_FILE);

        let replay = Replay::open(&replay_path).unwrap();

        assert_eq!(replay.gamemode, Gamemode::STD);
        assert_eq!(replay.game_version, 20210520);
        assert_eq!(replay.map_hash, "400751ddba867c309b16487d546dcfdd");
        assert_eq!(replay.player_name, "Sailor SnoW");
        assert_eq!(replay.replay_hash, "caf14311cabb3a6b67697d96db5e7824");
        assert_eq!(replay.number_300s, 592);
        assert_eq!(replay.number_100s, 2);
        assert_eq!(replay.number_50s, 0);
        assert_eq!(replay.number_gekis, 140);
        assert_eq!(replay.number_katus, 2);
        assert_eq!(replay.number_misses, 0);
        assert_eq!(replay.total_score, 13392443);
        assert_eq!(replay.greatest_combo, 852);
        assert_eq!(replay.is_full_combo, true);
        assert_eq!(replay.mods, Mods::HIDDEN);
        assert_eq!(replay.life_bar_graph, Some("".to_string()));
        assert_eq!(
            replay.play_date.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2021-07-08 18:26:50"
        );
        assert_eq!(replay.score_id, 3760034870);

        assert_eq!(replay.replay_data.seed, Some(19290764));
    }

    #[test]
    #[ignore]
    fn write_replay() {
        let replay_path = Path::new(TEST_REPLAY_FILE);

        let replay = Replay::open(&replay_path).unwrap();

        replay.write(Path::new(TEST_NEW_REPLAY_FILE)).unwrap();
    }
}
