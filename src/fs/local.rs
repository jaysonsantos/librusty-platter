use std::path::Path;
use std::fs::create_dir;

use fs::Filesystem;
use result::RustyPlatterResult;

pub struct LocalFileSystem<'a> {
    base_path: &'a Path,
}

impl<'a> LocalFileSystem<'a> {
    pub fn new(base_path: &'a str) -> Self {
        LocalFileSystem { base_path: Path::new(base_path) }
    }
}

impl<'a> Filesystem for LocalFileSystem<'a> {
    fn mkdir(&self, path: &str) -> RustyPlatterResult<()> {
        Ok(create_dir(self.base_path.join(path))?)
    }
    fn mv(&self, from: &str, to: &str) -> RustyPlatterResult<()> {
        unimplemented!()
    }
    fn rm(&self, path: &str) -> RustyPlatterResult<()> {
        unimplemented!()
    }
    fn exists(&self, path: &str) -> bool {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use self::tempdir::TempDir;

    use ::fs::Filesystem;
    use ::fs::local::LocalFileSystem;

    #[test]
    fn test_mkdir() {
        let temp = TempDir::new("test_mkdir").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());
        fs.mkdir("test");
        assert!(path.join("test").exists());
    }
}
