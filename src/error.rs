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

    #[error("Invalid gamemode replay value")]
    InvalidGamemode,
    #[error("Unknown error while reading string")]
    CantReadString,
    #[error("The byte value read was unexpected for this field")]
    UnexpectedFullComboValue,
    #[error("Error while reading the first string part")]
    UnexpectedStringValue,
}
