use super::*;
use crate::utils::lzma::compress_replay_data;

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

impl From<&ReplayData> for String {
    fn from(replay_data: &ReplayData) -> Self {
        let mut s = String::new();

        // default first frames in each replay
        for frame in replay_data.frames.iter() {
            let frame_string: String = frame.into();
            s.push_str(&frame_string);
            s.push(',');
        }

        match replay_data.seed {
            Some(seed) => {
                s.push_str("-12345|0|0|");
                s.push_str(&seed.to_string());
                s.push(',');
                s
            }
            None => s,
        }
    }
}

impl TryFrom<&ReplayData> for Vec<u8> {
    type Error = Error;

    fn try_from(replay_data: &ReplayData) -> Result<Self, Error> {
        let uncompressed = String::from(replay_data).as_bytes().to_vec();
        compress_replay_data(uncompressed)
    }
}

impl ReplayData {
    pub fn new() -> Self {
        Self::default()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/// Parsed data of a frame replay data
#[derive(Debug, Clone, Copy, Default)]
pub struct ReplayFrame {
    /// Time in milliseconds since the previous action
    pub w: Long,
    /// x-coordinate of the cursor from 0 - 512
    pub x: Float,
    /// y-coordinate of the cursor from 0 - 384
    pub y: Float,
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

        Ok(frame)
    }
}

impl From<&ReplayFrame> for String {
    fn from(frame: &ReplayFrame) -> Self {
        format!("{}|{}|{}|{}", frame.w, frame.x, frame.y, frame.z)
    }
}

impl ReplayFrame {
    pub fn new() -> Self {
        Self::default()
    }
}
