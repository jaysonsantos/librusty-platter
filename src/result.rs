use std::io;

pub type RustyPlatterResult<T> = Result<T, Error>;

pub enum Error {
    IoError(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}
