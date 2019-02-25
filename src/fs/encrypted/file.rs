use crate::fs::encrypted::{EncryptedFs, NONCE_SIZE, TAG_SIZE};
use crate::fs::File;
use std::io::{self, Write};
use std::iter;

pub const USER_BLOCK_SIZE: usize = 64 * 1024;
pub const BLOCK_SIZE: usize = USER_BLOCK_SIZE + NONCE_SIZE + TAG_SIZE;

// TODO: Decrease the amount of copy operations

pub struct EncryptedFile<'a, 'b> {
    filesystem: &'a EncryptedFs<'b>,
    file: Box<File>,
    buffer: [u8; BLOCK_SIZE],
    buffer_filled_length: usize,
    current_nonce: [u8; NONCE_SIZE],
}

impl<'a, 'b> EncryptedFile<'a, 'b> {
    pub fn new(file: Box<File>, filesystem: &'a EncryptedFs<'b>) -> Self {
        EncryptedFile {
            filesystem,
            file,
            buffer: [0; BLOCK_SIZE],
            buffer_filled_length: 0,
            current_nonce: [0; NONCE_SIZE],
        }
    }

    fn write_chunk(&mut self, chunk: &[u8]) -> io::Result<usize> {
        if self.buffer_filled_length != 0 {
            let bytes_to_write = (USER_BLOCK_SIZE - self.buffer_filled_length).min(chunk.len());
            let (difference, rest) = chunk.split_at(bytes_to_write);
            let (buffer, _) = self.buffer[self.buffer_filled_length..].split_at_mut(bytes_to_write);
            buffer.copy_from_slice(difference);
            self.buffer_filled_length += bytes_to_write;

            if self.buffer_filled_length >= USER_BLOCK_SIZE {
                self.filesystem.random.fill(&mut self.current_nonce);
                let encrypted_data = self
                    .filesystem
                    .encrypt_data(&self.buffer, &self.current_nonce)
                    .unwrap();
                assert_eq!(self.file.write(&encrypted_data)?, BLOCK_SIZE);
                let (buffer, _) = self.buffer.split_at_mut(rest.len());
                buffer.copy_from_slice(rest);
                self.buffer_filled_length = rest.len();
            }
            return Ok(chunk.len());
        }

        if chunk.len() < USER_BLOCK_SIZE {
            let (buffer, _) = self.buffer.split_at_mut(chunk.len());
            buffer.copy_from_slice(chunk);
            self.buffer_filled_length = chunk.len();
            return Ok(chunk.len());
        }

        let buffer = &mut self.buffer[..USER_BLOCK_SIZE];
        buffer.copy_from_slice(chunk);
        self.filesystem.random.fill(&mut self.current_nonce);
        let encrypted_data = self
            .filesystem
            .encrypt_data(&self.buffer, &self.current_nonce)
            .unwrap();
        assert_eq!(self.file.write(&encrypted_data)?, BLOCK_SIZE);
        Ok(chunk.len())
    }
}

impl<'a, 'b> Write for EncryptedFile<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut written_data = 0;
        for chunk in buf.chunks(USER_BLOCK_SIZE) {
            written_data += self.write_chunk(chunk)?;
        }
        Ok(written_data)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::fs::local::LocalFileSystem;
    use crate::Config;
    use crate::EncryptedFs;

    use ring::test::rand::FixedByteRandom;
    use tempdir::TempDir;

    const PASSWORD: &'static str = "password";
    const ITERATIONS: u32 = 10_000;

    #[test]
    fn test_create() {
        let _ = env_logger::try_init();
        let temp = TempDir::new("test_create").unwrap();
        let path = temp.path();
        let fs = dbg!(LocalFileSystem::new(path.to_str().unwrap()));
        let config = dbg!(Config::new_with_custom_random(
            PASSWORD,
            ITERATIONS,
            &fs,
            Box::new(FixedByteRandom { byte: 0 }),
        ))
        .unwrap();
        let encrypted =
            EncryptedFs::with_custom_random(&fs, config, Box::new(FixedByteRandom { byte: 0 }));
        let mut f = encrypted.create("abc").unwrap();
        assert_eq!(f.write("he".as_bytes()).unwrap(), 2);
        assert_eq!(f.write("llo".as_bytes()).unwrap(), 3);
        assert_eq!(
            f.write(" ".repeat(64 * 1024 - 5).as_bytes()).unwrap(),
            64 * 1024 - 5
        );
        // assert_eq!(f.write(" ".repeat(64).as_bytes()).unwrap(), 64);
        dbg!(temp.path());
        // f.flush().unwrap();
        // ::std::thread::sleep(::std::time::Duration::from_secs(60));
        assert!(encrypted.exists("abc"));
    }
}
