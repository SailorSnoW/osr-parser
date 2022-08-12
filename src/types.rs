use crate::error::Error;
use bitflags::bitflags;
use core::default::Default;

// Data types
pub type Byte = u8;
pub type Short = u16;
pub type Integer = u32;
pub type Long = i64;
pub type Float = f32;

// Format types
#[derive(Debug, PartialEq, Eq)]
pub enum Gamemode {
    STD,
    TAIKO,
    CTB,
    MANIA,
}

impl Default for Gamemode {
    fn default() -> Self {
        Self::STD
    }
}

impl From<&Gamemode> for u8 {
    fn from(gamemode: &Gamemode) -> Self {
        match gamemode {
            Gamemode::STD => 0,
            Gamemode::TAIKO => 1,
            Gamemode::CTB => 2,
            Gamemode::MANIA => 3,
        }
    }
}

impl TryFrom<Byte> for Gamemode {
    type Error = Error;

    fn try_from(value: Byte) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::STD),
            0x01 => Ok(Self::TAIKO),
            0x02 => Ok(Self::CTB),
            0x03 => Ok(Self::MANIA),
            _ => Err(Error::InvalidGamemode),
        }
    }
}

bitflags! {
    #[derive(Default)]
    pub struct Mods: u32 {
        const NONE           = 0;
        const NO_FAIL         = 1;
        const EASY           = 2;
        const TOUCH_DEVICE    = 4;
        const HIDDEN         = 8;
        const HARDROCK       = 16;
        const SUDDEN_DEATH    = 32;
        const DOUBLETIME     = 64;
        const RELAX          = 128;
        const HALFTIME       = 256;
        const NIGHTCORE      = 512; // Only set along with DoubleTime. i.e: NC only gives 576
        const FLASHLIGHT     = 1024;
        const AUTOPLAY       = 2048;
        const SPUN_OUT        = 4096;
        const RELAX2         = 8192;    // Autopilot
        const PERFECT        = 16384; // Only set along with SuddenDeath. i.e: PF only gives 16416
        const KEY4           = 32768;
        const KEY55           = 65536;
        const KEY6           = 13107;
        const KEY7           = 262144;
        const KEY8           = 524288;
        const FADE_IN         = 1048576;
        const RANDOM         = 2097152;
        const CINEMA         = 4194304;
        const TARGET         = 8388608;
        const KEY0          = 16777216;
        const KEY_COOP        = 33554432;
        const KEY1           = 67108864;
        const KEY3           = 134217728;
        const KEY2           = 268435456;
        const SCORE_V2        = 536870912;
        const MIRROR         = 1073741824;
    }
}

impl From<Integer> for Mods {
    fn from(value: Integer) -> Self {
        Self::from_bits_truncate(value)
    }
}

impl From<Mods> for Integer {
    fn from(mods: Mods) -> Self {
        mods.bits()
    }
}
