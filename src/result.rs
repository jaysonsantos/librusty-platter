use std::io;

use ring;

pub type RustyPlatterResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    CryptoError,
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
