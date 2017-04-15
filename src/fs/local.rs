use fs;
use result::RustyPlatterResult;

use std::fs::{create_dir, remove_dir_all, remove_file, rename};
use std::fs::File as StdFile;
use std::io::Write;
use std::io::Result as IoResult;
use std::path::{Path, MAIN_SEPARATOR};

/// Implementation of a local filesystem
pub struct LocalFileSystem<'a> {
    base_path: &'a Path,
}

impl<'a> LocalFileSystem<'a> {
    pub fn new(base_path: &'a str) -> Self {
        LocalFileSystem { base_path: Path::new(base_path) }
    }
}

impl<'a> fs::Filesystem for LocalFileSystem<'a> {
    fn path_separator(&self) -> String {
        let mut sep = String::new();
        sep.push(MAIN_SEPARATOR);
        sep
    }
    fn mkdir(&self, path: &str) -> RustyPlatterResult<()> {
        Ok(create_dir(self.base_path.join(path))?)
    }
    fn mv(&self, from: &str, to: &str) -> RustyPlatterResult<()> {
        Ok(rename(self.base_path.join(from), self.base_path.join(to))?)
    }
    fn rm(&self, path: &str) -> RustyPlatterResult<()> {
        let path = self.base_path.join(path);
        if path.is_dir() {
            Ok(remove_dir_all(path)?)
        } else {
            Ok(remove_file(path)?)
        }
    }
    fn exists(&self, path: &str) -> bool {
        self.base_path.join(path).exists()
    }
    fn open(&self, path: &str) -> RustyPlatterResult<Box<fs::File>> {
        Ok(LocalFile::open_boxed(path)?)
    }
    fn create(&self, path: &str) -> RustyPlatterResult<Box<fs::File>> {
        Ok(LocalFile::create_boxed(path)?)
    }
}

pub struct LocalFile {
    fd: StdFile,
}

impl LocalFile {
    pub fn open(path: &str) -> RustyPlatterResult<Self> {
        Ok(LocalFile { fd: StdFile::open(path)? })
    }
    pub fn open_boxed(path: &str) -> RustyPlatterResult<Box<Self>> {
        Ok(Box::new(LocalFile::open(path)?))
    }
    pub fn create(path: &str) -> RustyPlatterResult<Self> {
        Ok(LocalFile { fd: StdFile::create(path)? })
    }
    pub fn create_boxed(path: &str) -> RustyPlatterResult<Box<Self>> {
        Ok(Box::new(LocalFile::create(path)?))
    }
}

impl fs::File for LocalFile {}

impl Write for LocalFile {
    fn write(&mut self, content: &[u8]) -> IoResult<usize> {
        Ok(self.fd.write(content)?)
    }
    fn flush(&mut self) -> IoResult<()> {
        Ok(self.fd.flush()?)
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use self::tempdir::TempDir;

    use super::*;
    use super::fs::*;
    use std::fs as std_fs;

    #[test]
    fn test_mkdir() {
        let temp = TempDir::new("test_mkdir").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());
        fs.mkdir("test").unwrap();
        assert!(path.join("test").exists());
    }

    #[test]
    fn test_exists() {
        let temp = TempDir::new("test_mkdir").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());
        assert!(fs.exists("."));
        assert!(!fs.exists("abc"));
    }

    #[test]
    fn test_rm() {
        let temp = TempDir::new("test_mkdir").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());

        std_fs::create_dir(path.join("dir")).unwrap();
        fs.rm("dir").unwrap();
        std_fs::File::create(path.join("file")).unwrap();
        fs.rm("file").unwrap();
    }

    #[test]
    fn test_mv() {
        let temp = TempDir::new("test_mkdir").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());

        std_fs::create_dir(path.join("dir")).unwrap();
        fs.mv("dir", "dir2").unwrap();
        assert!(path.join("dir2").exists());

        std_fs::File::create(path.join("file")).unwrap();
        fs.mv("file", "file2").unwrap();
        assert!(path.join("file2").exists());
    }

    #[test]
    fn test_write() {
        let temp = TempDir::new("test_mkdir").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());
        let mut file = fs.create("ab.txt").unwrap();
        assert_eq!(file.write(b"a").unwrap(), 1 as usize);
    }
}
