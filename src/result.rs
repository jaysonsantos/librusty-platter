pub enum Error {
    PermissionDenied,
}

pub type RustyPlatterResult<T> = Result<T, Error>;
