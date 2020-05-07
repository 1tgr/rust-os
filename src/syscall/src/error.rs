use core::convert::TryFrom;
use core::result;

#[derive(Debug, Eq, PartialEq)]
pub enum ErrNum {
    Utf8Error,
    OutOfMemory,
    InvalidHandle,
    NotSupported,
    FileNotFound,
    InvalidArgument,
}

impl TryFrom<usize> for ErrNum {
    type Error = ();

    fn try_from(value: usize) -> result::Result<Self, ()> {
        match value {
            1 => Ok(Self::Utf8Error),
            2 => Ok(Self::OutOfMemory),
            3 => Ok(Self::InvalidHandle),
            4 => Ok(Self::NotSupported),
            5 => Ok(Self::FileNotFound),
            6 => Ok(Self::InvalidArgument),
            _ => Err(()),
        }
    }
}

impl Into<usize> for ErrNum {
    fn into(self) -> usize {
        match self {
            Self::Utf8Error => 1,
            Self::OutOfMemory => 2,
            Self::InvalidHandle => 3,
            Self::NotSupported => 4,
            Self::FileNotFound => 5,
            Self::InvalidArgument => 6,
        }
    }
}

pub type Result<T> = result::Result<T, ErrNum>;
