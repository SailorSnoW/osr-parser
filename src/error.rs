use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Given path is not a file: {}", path)]
    NotAFile { path: String },
    #[error("Given file is not an osu replay files (.osr): {}", file)]
    NotAReplayFile { file: String },
    #[error("Unknown error during file opening")]
    CantOpenFile,
    #[error("Unknown error while buffering the file replay datas")]
    FileBufferingError,
    #[error("Unexpected error while reading the value into buffer")]
    ReadBufferingError,

    #[error("Invalid gamemode replay value")]
    InvalidGamemode,
    #[error("Unknown error while reading string")]
    CantReadString,
    #[error("The byte value read was unexpected for this field")]
    UnexpectedFullComboValue,
    #[error("Error while reading the first string part")]
    UnexpectedStringValue,

    #[error("Unknown error while decompressing replay data")]
    ReplayDataDecompressError,

    #[error("Invalid event string format")]
    InvalidStringFrameFormat,
    #[error("Error while parsing a replay event value")]
    CantParseFrameValue,
    #[error("Event value 'x' isn't in the valid range 0 - 512")]
    InvalidFrameValueX,
    #[error("Event value 'y' isn't in the valid range 0 - 384")]
    InvalidFrameValueY,
}
