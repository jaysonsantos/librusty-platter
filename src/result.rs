use base64;

use ring;

use std::io;

pub type RustyPlatterResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    CryptoError,
    Base64Error,
    InvalidEncodedName,
    InvalidPathName,
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

impl From<base64::Base64Error> for Error {
    // TODO: Better error handling for base64
    fn from(_: base64::Base64Error) -> Self {
        Error::Base64Error
    }
}
