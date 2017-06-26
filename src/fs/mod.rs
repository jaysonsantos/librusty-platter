pub mod local;
pub mod encrypted;

use result::RustyPlatterResult;
use std::io::Write;

pub trait File: Write {}

/// Basic filesystem trait
pub trait Filesystem {
    fn path_separator(&self) -> String;
    fn mkdir(&self, path: &str) -> RustyPlatterResult<()>;
    fn mv(&self, from: &str, to: &str) -> RustyPlatterResult<()>;
    fn rm(&self, path: &str) -> RustyPlatterResult<()>;
    fn exists(&self, path: &str) -> bool;
    fn open(&self, path: &str) -> RustyPlatterResult<Box<File>>;
    fn create(&self, path: &str) -> RustyPlatterResult<Box<File>>;
}
