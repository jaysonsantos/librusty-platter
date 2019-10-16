pub mod encrypted;
pub mod local;

use crate::result::Result;
use std::io::Write;

pub trait File: Write {}

/// Basic filesystem trait
pub trait Filesystem {
    fn path_separator(&self) -> String;
    fn mkdir(&self, path: &str) -> Result<()>;
    fn mv(&self, from: &str, to: &str) -> Result<()>;
    fn rm(&self, path: &str) -> Result<()>;
    fn exists(&self, path: &str) -> bool;
    fn open(&self, path: &str) -> Result<Box<dyn File>>;
    fn create(&self, path: &str) -> Result<Box<dyn File>>;
}
