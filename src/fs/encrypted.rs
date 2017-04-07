use base64::{encode, decode};

use config::Config;
use fs::Filesystem;
use result::{RustyPlatterResult, Error};

use ring::aead::{SealingKey, OpeningKey, CHACHA20_POLY1305, seal_in_place, open_in_place};
use ring::pbkdf2;
use ring::rand::{SystemRandom, SecureRandom};

use serde_json;

/// Struct that deals with a `Filesystem` implementation writing encrypted and reading decrypted.
pub struct EncryptedFs<'a> {
    fs: &'a Filesystem,
    config: Config,
    random: Box<SecureRandom>,
}

impl<'a> EncryptedFs<'a> {
    fn new(fs: &'a Filesystem, config: Config) -> Self {
        EncryptedFs {
            fs: fs,
            config: config,
            random: Box::new(SystemRandom::new()),
        }
    }

    fn with_custom_random(fs: &'a Filesystem, config: Config, random: Box<SecureRandom>) -> Self {
        EncryptedFs {
            fs: fs,
            config: config,
            random: random,
        }
    }

    fn encrypt_name(&self, name: &str) -> RustyPlatterResult<String> {
        let keys = self.config.keys.as_ref().unwrap();
        let additional_data = [];
        let mut nonce = vec![0; keys.sealing.algorithm().nonce_len()];
        let mut output: Vec<u8> = vec![];
        let mut to_encrypt = name.as_bytes().to_vec();

        if to_encrypt.len() == 0 {
            return Err(Error::InvalidPathName);
        }

        // Initialize space for the tag
        for _ in 0..keys.sealing.algorithm().tag_len() {
            to_encrypt.push(0);
        }

        // Fill nonce with random data
        self.random.fill(&mut nonce)?;
        output.extend_from_slice(&nonce);

        let out = seal_in_place(&keys.sealing,
                                &nonce,
                                &additional_data,
                                &mut to_encrypt,
                                CHACHA20_POLY1305.tag_len())?;

        output.extend_from_slice(&to_encrypt);
        assert_eq!(to_encrypt.len(), out);
        Ok(encode(&output))
    }

    fn decrypt_name(&self, name: &str) -> RustyPlatterResult<String> {
        let keys = self.config.keys.as_ref().unwrap();
        let mut nonce = decode(name)?;
        let mut encrypted_data = nonce.split_off(keys.opening.algorithm().nonce_len());
        let additional_data = [];
        let decrypted = open_in_place(&keys.opening,
                                      &nonce,
                                      &additional_data,
                                      0,
                                      &mut encrypted_data).map_err(|_| Error::InvalidEncodedName)?;
        String::from_utf8(decrypted.to_vec()).map_err(|_| Error::InvalidEncodedName)
    }

    fn mkdir() {
        unimplemented!()
    }
}


#[cfg(test)]
mod tests {
    extern crate ring;
    extern crate tempdir;

    use fs::local::LocalFileSystem;

    use ring::aead::*;
    use ring::error::Unspecified;
    use ring::pbkdf2::*;
    use ring::rand::SystemRandom;

    use self::tempdir::TempDir;
    use std::io::Write;

    use super::*;

    const PASSWORD: &'static str = "password";
    const ITERATIONS: u32 = 100;

    struct DumbRandom {}

    impl DumbRandom {
        fn dumb_data(&self) -> Vec<u8> {
            (0..100).collect()
        }
    }

    impl SecureRandom for DumbRandom {
        fn fill(&self, mut buf: &mut [u8]) -> Result<(), Unspecified> {
            buf.write(&self.dumb_data()).unwrap();
            Ok(())
        }
    }

    #[test]
    fn test_en_decrypt_name() {
        let temp = TempDir::new("test_en_decrypt_name").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());

        let config = Config::new(PASSWORD, ITERATIONS, &fs).unwrap();
        let encrypted = EncryptedFs::with_custom_random(&fs, config, Box::new(DumbRandom {}));
        let data = "path name";
        let encrypted_name = encrypted.encrypt_name(data).unwrap();
        assert_eq!(encrypted.decrypt_name(&*encrypted_name).unwrap(), data);
    }
}
