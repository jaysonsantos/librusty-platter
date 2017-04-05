use base64::{encode, decode};

use config::Config;
use fs::Filesystem;
use result::{RustyPlatterResult, Error};

use ring::aead::{SealingKey, OpeningKey, CHACHA20_POLY1305, seal_in_place, open_in_place};
use ring::pbkdf2;
use ring::rand::{SystemRandom, SecureRandom};

use serde_json;

const MINIMUM_ENCRYPTION_SIZE: usize = 16;

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
        if let Some(remaining) = MINIMUM_ENCRYPTION_SIZE.checked_sub(to_encrypt.len()) {
            for _ in 0..remaining {
                to_encrypt.push(0);
            }
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
        println!("{}", encode(&output));
        Ok(encode(&output))
    }

    fn decrypt_name(&self, name: &str) -> RustyPlatterResult<String> {
        let keys = self.config.keys.as_ref().unwrap();
        let mut nonce = decode(name)?;
        let mut encrypted_data = nonce.split_off(keys.opening.algorithm().nonce_len());
        println!("{:?} {:?} {}", nonce, encrypted_data, encrypted_data.len());
        let additional_data = [];
        open_in_place(&keys.opening,
                      &nonce,
                      &additional_data,
                      0,
                      &mut encrypted_data).map_err(|_| Error::InvalidEncodedName)?;
        String::from_utf8(encrypted_data).map_err(|_| Error::InvalidEncodedName)
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
    fn test_encrypt_name() {
        let temp = TempDir::new("test_mkdir").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());

        let config = Config::new(PASSWORD, ITERATIONS, &fs).unwrap();
        let encrypted = EncryptedFs::with_custom_random(&fs, config, Box::new(DumbRandom {}));
        // let a = "a nice string to encrypt for test";
        let a = "a";
        let encrypted_name = encrypted.encrypt_name(a).unwrap();
        let encrypted_name2 = encrypted.encrypt_name(a).unwrap();
        assert_eq!(encrypted_name, encrypted_name2);
        assert_eq!(encrypted.decrypt_name(&*encrypted_name).unwrap(), a);

    }

    #[test]
    fn test_decrypt_name() {
        let temp = TempDir::new("test_mkdir").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());

        let config = Config::new(PASSWORD, ITERATIONS, &fs).unwrap();
        let encrypted = EncryptedFs::with_custom_random(&fs, config, Box::new(DumbRandom {}));
        let encrypted_name = encrypted.decrypt_name("AAECAwQFBgcICQoLPMcm4S+zB9x3QDyUx89jWA==")
            .unwrap();

        assert_eq!(encrypted_name, "abc");
    }

    #[test]
    fn test_mkdir() {
        let temp = TempDir::new("test_mkdir").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());

        let config = Config::new(PASSWORD, ITERATIONS, &fs).unwrap();
        let encrypted = EncryptedFs::new(&fs, config);

        // encrypted.mkdir("abc");
    }

    #[test]
    fn test_encryption() {
        // The password will be used to generate a key
        let password = b"nice password";

        // Usually the salt has some random data and something that relates to the user
        // like an username
        let salt = [0, 1, 2, 3, 4, 5, 6, 7];

        // Keys are sent as &[T] and must have 32 bytes
        let mut key = [0; 32];
        derive(&HMAC_SHA256, 100, &salt, &password[..], &mut key);

        // Your private data
        let content = b"content to encrypt".to_vec();
        println!("Content to encrypt's size {}", content.len());

        // Additional data that you would like to send and it would not be encrypted but it would
        // be signed
        let additional_data: [u8; 0] = [];

        // Ring uses the same input variable as output
        let mut in_out = content.clone();

        // The input/output variable need some space for a suffix
        println!("Tag len {}", CHACHA20_POLY1305.tag_len());
        for _ in 0..CHACHA20_POLY1305.tag_len() {
            in_out.push(0);
        }

        // Opening key used to decrypt data
        let opening_key = OpeningKey::new(&CHACHA20_POLY1305, &key).unwrap();

        // Sealing key used to encrypt data
        let sealing_key = SealingKey::new(&CHACHA20_POLY1305, &key).unwrap();

        // Random data must be used only once per encryption
        let mut nonce = vec![0; 12];

        // Fill nonce with random data
        let rand = SystemRandom::new();
        //        rand.fill(&mut nonce).unwrap();

        // Encrypt data into in_out variable
        let output_size = seal_in_place(&sealing_key,
                                        &nonce,
                                        &additional_data,
                                        &mut in_out,
                                        CHACHA20_POLY1305.tag_len())
            .unwrap();

        println!("Encrypted data's size {} {:?}", output_size, in_out);

        let decrypted_data = open_in_place(&opening_key, &nonce, &additional_data, 0, &mut in_out)
            .unwrap();

        println!("{:?}", String::from_utf8(decrypted_data.to_vec()).unwrap());
        assert_eq!(content, decrypted_data);
        assert!(false);
    }
}
