pub mod local;
pub mod encrypted;

use result::RustyPlatterResult;

pub trait File {
    fn write(&self, content: &[u8]) -> RustyPlatterResult<usize>;
}

/// Basic filesystem trait
pub trait Filesystem {
    fn mkdir(&self, path: &str) -> RustyPlatterResult<()>;
    fn mv(&self, from: &str, to: &str) -> RustyPlatterResult<()>;
    fn rm(&self, path: &str) -> RustyPlatterResult<()>;
    fn exists(&self, path: &str) -> bool;
    fn open(&self, path: &str) -> RustyPlatterResult<Box<File>>;
}
