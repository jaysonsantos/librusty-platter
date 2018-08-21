use config::Config;
use data_encoding::BASE32;
use fs::Filesystem;
use result::{ErrorKind, Result, ResultExt};
use ring::aead::{open_in_place, seal_in_place, CHACHA20_POLY1305};
use ring::rand::{SecureRandom, SystemRandom};

/// Struct that deals with a `Filesystem` implementation writing encrypted and reading decrypted.
pub struct EncryptedFs<'a> {
    fs: &'a Filesystem,
    config: Config,
    random: Box<SecureRandom>,
}

impl<'a> EncryptedFs<'a> {
    #![allow(dead_code)]
    fn new(fs: &'a Filesystem, config: Config) -> Self {
        EncryptedFs {
            fs: fs,
            config: config,
            random: Box::new(SystemRandom::new()),
        }
    }

    #[allow(dead_code)]
    fn with_custom_random(fs: &'a Filesystem, config: Config, random: Box<SecureRandom>) -> Self {
        // Constructor mainly used for tests where we can mock random values
        EncryptedFs {
            fs: fs,
            config: config,
            random: random,
        }
    }

    /// Encrypt a name and return it as base64 string
    pub fn encrypt_name(&self, name: &str) -> Result<String> {
        let sealing_key = self.config.sealing_key();
        let mut nonce = vec![0; sealing_key.algorithm().nonce_len()];
        self.random.fill(&mut nonce)?;
        Ok(BASE32.encode(&self.encrypt_data(name.as_bytes(), &*nonce)?))
    }

    /// Encrypt already chunked slices returning a binary vector with it's nonce (12 bytes)
    /// and encrypted data (input_data.len())
    pub fn encrypt_data(&self, input_data: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
        let additional_data = [];
        let sealing_key = self.config.sealing_key();
        let mut output: Vec<u8> = vec![];
        let mut to_encrypt = input_data.to_vec();

        if to_encrypt.is_empty() {
            bail!(ErrorKind::InvalidPathName("".to_owned()));
        }

        // Initialize space for the tag
        let tag_len = sealing_key.algorithm().tag_len();
        to_encrypt.resize(input_data.len() + tag_len, 0);

        output.extend_from_slice(&nonce);

        // Don't truncate because we want to keep it as fixed size
        seal_in_place(
            &sealing_key,
            &nonce,
            &additional_data,
            &mut to_encrypt,
            CHACHA20_POLY1305.tag_len(),
        )?;

        output.extend_from_slice(&to_encrypt);
        Ok(output)
    }

    /// Decrypt a base64 encoded string returning a string
    pub fn decrypt_name(&self, name: &str) -> Result<String> {
        let data = BASE32.decode(name.as_bytes())?;
        let decrypted = self.decrypt_data(&*data)?;
        String::from_utf8(decrypted.to_vec()).chain_err(|| ErrorKind::InvalidEncodedName)
    }

    /// Decrypt already chunked slices and return the binary data
    pub fn decrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        let opening_key = self.config.opening_key();
        let mut nonce = data.to_vec();
        let mut encrypted_data = nonce.split_off(opening_key.algorithm().nonce_len());
        let additional_data = [];
        let decrypted = open_in_place(
            &opening_key,
            &nonce,
            &additional_data,
            0,
            &mut encrypted_data,
        ).chain_err(|| ErrorKind::InvalidEncodedName)?;
        Ok(decrypted.to_vec())
    }

    fn encrypt_path(&self, name: &str) -> Result<String> {
        let path_sep = self.fs.path_separator();
        let path: Vec<&str> = name.split(&*path_sep)
            // Remove stuff like a//b
            .filter(|name| !name.is_empty())
            .collect();
        let mut encrypted_path = vec![];
        for part in &path {
            encrypted_path.push(self.encrypt_name(part)?);
        }
        Ok(encrypted_path.join(&*path_sep))
    }

    /// Create an encrypted directory
    pub fn mkdir(&self, name: &str) -> Result<()> {
        let encrypted_path = self.encrypt_path(name)?;
        self.fs.mkdir(&*encrypted_path)
    }

    /// Check if a path exists
    pub fn exists(&self, name: &str) -> bool {
        if let Ok(encrypted_path) = self.encrypt_path(name) {
            self.fs.exists(&*encrypted_path)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate env_logger;
    extern crate ring;
    extern crate tempdir;

    use self::tempdir::TempDir;
    use super::*;
    use fs::local::LocalFileSystem;
    use ring::error::Unspecified;
    use std::io::Write;

    const PASSWORD: &'static str = "password";
    const ITERATIONS: u32 = 10_000;

    struct DumbRandom {}

    impl DumbRandom {
        fn dumb_data(&self) -> Vec<u8> {
            (0..100).collect()
        }
    }

    impl SecureRandom for DumbRandom {
        fn fill(&self, mut buf: &mut [u8]) -> ::std::result::Result<(), Unspecified> {
            buf.write(&self.dumb_data()).unwrap();
            Ok(())
        }
    }

    #[test]
    fn test_en_decrypt_name() {
        let temp = TempDir::new("test_en_decrypt_name").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());

        let config =
            Config::new_with_custom_random(PASSWORD, ITERATIONS, &fs, Box::new(DumbRandom {}))
                .unwrap();
        let encrypted = EncryptedFs::with_custom_random(&fs, config, Box::new(DumbRandom {}));
        let data = "path name";
        let encrypted_name = encrypted.encrypt_name(data).unwrap();
        assert_eq!(encrypted.decrypt_name(&*encrypted_name).unwrap(), data);
    }

    #[test]
    fn test_mkdir() {
        let _ = env_logger::try_init();
        debug!("encrypted.rs test_mkdir");
        let temp = TempDir::new("test_mkdir").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());
        debug!("{:?}", fs);
        debug!("Start config");
        let config =
            Config::new_with_custom_random(PASSWORD, ITERATIONS, &fs, Box::new(DumbRandom {}))
                .unwrap();
        debug!("{:?}", config);
        let encrypted = EncryptedFs::with_custom_random(&fs, config, Box::new(DumbRandom {}));
        encrypted.mkdir("abc").unwrap();
        assert!(encrypted.exists("abc"));
    }

    #[test]
    fn test_exists() {
        let _ = env_logger::try_init();
        debug!("encrypted.rs test_exists");
        let temp = TempDir::new("test_exists").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());
        debug!("{:?}", fs);
        debug!("Start config");
        let config =
            Config::new_with_custom_random(PASSWORD, ITERATIONS, &fs, Box::new(DumbRandom {}))
                .unwrap();
        debug!("{:?}", config);
        let encrypted = EncryptedFs::with_custom_random(&fs, config, Box::new(DumbRandom {}));
        encrypted.mkdir("abc").unwrap();
        assert!(encrypted.exists("abc"));
    }
}
