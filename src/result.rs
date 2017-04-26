use data_encoding;

use ring;

use std::io;

pub type RustyPlatterResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    CryptoError,
    Base32Error,
    InvalidEncodedName,
    InvalidPathName,
    IterationsNumberTooSmall,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl From<ring::error::Unspecified> for Error {
    fn from(_: ring::error::Unspecified) -> Self {
        Error::CryptoError
    }
}

impl From<data_encoding::DecodeError> for Error {
    // TODO: Better error handling for base32
    fn from(_: data_encoding::DecodeError) -> Self {
        Error::Base32Error
    }
}
