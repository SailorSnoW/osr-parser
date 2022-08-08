use chrono::NaiveDateTime;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
    str::FromStr,
};
use xz2::stream::{Action, Stream};

use crate::error::Error;
use crate::types::*;
use crate::utils::read::{read_long, ReadResult};
use crate::utils::{read, ticks_to_datetime};

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Contains decompressed and parsed data of a replay
#[derive(Debug, Default)]
pub struct ReplayData {
    /// Parsed frames of the replay
    pub frames: Vec<ReplayFrame>,
    /// RNG seed used for the score
    /// Note: only available on replay file set on version '20130319' or later
    pub seed: Option<Integer>,
}

impl FromStr for ReplayData {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splitted_frames: Vec<&str> = s.split(',').collect();

        let mut seed = None;
        let mut frames: Vec<ReplayFrame> = Vec::new();

        for frame in splitted_frames.iter() {
            // seed check
            if frame.starts_with("-12345|0|0|") {
                seed = Some(
                    u32::from_str(frame.split('|').collect::<Vec<&str>>()[3])
                        .map_err(|_| Error::CantParseFrameValue)?,
                );
                break;
            }

            match ReplayFrame::from_str(frame) {
                Ok(f) => frames.push(f),
                Err(_) => (),
            }
        }

        Ok(Self { frames, seed })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Parsed data of a frame replay data
#[derive(Debug, Clone, Copy, Default)]
pub struct ReplayFrame {
    /// Time in milliseconds since the previous action
    pub w: Long,
    /// x-coordinate of the cursor from 0 - 512
    x: Float,
    /// y-coordinate of the cursor from 0 - 384
    y: Float,
    /// Bitwise combination of keys/mouse buttons pressed
    /// (M1 = 1, M2 = 2, K1 = 4, K2 = 8, Smoke = 16)
    /// (K1 is always used with M1; K2 is always used with M2: 1+4=5; 2+8=10)
    pub z: Integer,
}

impl FromStr for ReplayFrame {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splitted_event: Vec<&str> = s.split('|').collect();

        if splitted_event.len() != 4 {
            return Err(Error::InvalidStringFrameFormat);
        }

        let frame = Self {
            w: Long::from_str(splitted_event[0]).map_err(|_| Error::CantParseFrameValue)?,
            x: f32::from_str(splitted_event[1]).map_err(|_| Error::CantParseFrameValue)?,
            y: f32::from_str(splitted_event[2]).map_err(|_| Error::CantParseFrameValue)?,
            z: Integer::from_str(splitted_event[3]).map_err(|_| Error::CantParseFrameValue)?,
        };

        frame.validate_x()?;
        frame.validate_y()?;

        Ok(frame)
    }
}

impl ReplayFrame {
    pub fn set_x(&mut self, x: f32) -> Result<(), Error> {
        self.x = x;
        Self::validate_x(self)?;
        Ok(())
    }

    pub fn set_y(&mut self, y: f32) -> Result<(), Error> {
        self.y = y;
        Self::validate_y(self)?;
        Ok(())
    }

    fn validate_x(&self) -> Result<(), Error> {
        if self.x >= -512_f32 && self.x <= 512_f32 {
            Ok(())
        } else {
            return Err(Error::InvalidFrameValueX);
        }
    }

    fn validate_y(&self) -> Result<(), Error> {
        if self.y >= -384_f32 && self.y <= 384_f32 {
            Ok(())
        } else {
            return Err(Error::InvalidFrameValueY);
        }
    }
}

/// Structure of a replay containing parsed values
#[derive(Debug)]
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
    pub mods: Integer,
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
        let gamemode: Gamemode = Gamemode::try_from(read::read_byte(buf)?)?;

        let game_version = read::read_integer(buf)?;

        let map_hash = read::read_string(buf)?.unwrap_or_default();
        let player_name = read::read_string(buf)?.unwrap_or_default();
        let replay_hash = read::read_string(buf)?.unwrap_or_default();

        let number_300s = read::read_short(buf)?;
        let number_100s = read::read_short(buf)?;
        let number_50s = read::read_short(buf)?;
        let number_gekis = read::read_short(buf)?;
        let number_katus = read::read_short(buf)?;
        let number_misses = read::read_short(buf)?;

        let total_score = read::read_integer(buf)?;
        let greatest_combo = read::read_short(buf)?;

        let is_full_combo = match read::read_byte(buf)? {
            0x00 => false,
            0x01 => true,
            _ => return Err(Error::UnexpectedFullComboValue),
        };

        let mods = read::read_integer(buf)?;

        let life_bar_graph = read::read_string(buf)?;
        let play_date = Self::read_play_date(buf)?;
        let compressed_length = read::read_integer(buf)?;

        let mut compressed_replay_data = vec![0u8; compressed_length as usize];
        buf.read(&mut compressed_replay_data)
            .map_err(|_| Error::ReadBufferingError)?;

        let decompressed_replay_data = Self::decompress_replay_data(&compressed_replay_data)?;
        let replay_data =
            ReplayData::from_str(&String::from_utf8(decompressed_replay_data).unwrap_or_default())?;

        let score_id = read::read_long(buf)?;

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

    fn read_play_date<R: Read>(buf: &mut R) -> ReadResult<NaiveDateTime> {
        let timestamp_ticks = read_long(buf)?;
        Ok(ticks_to_datetime(timestamp_ticks))
    }

    fn decompress_replay_data(compressed_data: &Vec<u8>) -> Result<Vec<u8>, Error> {
        let buffer = &compressed_data.as_slice();
        let mut s = Vec::with_capacity(u32::MAX as usize);

        let mut lzma_decoder = Stream::new_lzma_decoder(u32::MAX as u64).unwrap();

        lzma_decoder
            .process_vec(buffer, &mut s, Action::Finish)
            .unwrap();
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Read},
    };

    use super::{Gamemode, Replay};

    const TEST_REPLAY_FILE: &'static str = "./assets/examples/replay-test.osr";

    #[test]
    fn parse_from_buffer() {
        let file = File::open(TEST_REPLAY_FILE).unwrap();

        let mut buffer = Vec::new();
        let mut reader = BufReader::new(file);

        reader.read_to_end(&mut buffer).unwrap();

        let replay = Replay::from_buffer(&mut buffer.as_slice()).unwrap();

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
        assert_eq!(replay.mods, 8);
        assert_eq!(replay.life_bar_graph, None);
        assert_eq!(
            replay.play_date.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2021-07-08 18:26:50"
        );
        assert_eq!(replay.score_id, 3760034870);

        assert_eq!(replay.replay_data.seed, Some(19290764));
    }
}
