use result::RustyPlatterResult;

/// Basic filesystem trait
pub trait Filesystem {
    fn mkdir(&self) -> RustyPlatterResult<()>;
    fn mv(&self, from: &str, to: &str) -> RustyPlatterResult<()>;
    fn rm(&self, dir_or_file: &str) -> RustyPlatterResult<()>;
}
