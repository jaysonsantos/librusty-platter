use base64::encode;

use ring::aead::{SealingKey, OpeningKey, CHACHA20_POLY1305, seal_in_place};
use ring::pbkdf2;
use ring::rand::SystemRandom;

use serde_json;

use fs::Filesystem;
use result::RustyPlatterResult;

struct Keys {
    opening: OpeningKey,
    sealing: SealingKey,
}

#[derive(Serialize, Deserialize)]
struct Config {
    salt: [u8; 16],

    iterations: u32,

    #[serde(skip_serializing, skip_deserializing)]
    keys: Option<Keys>,
}

impl Config {
    fn new(password: &str, iterations: u32, fs: &Filesystem) -> RustyPlatterResult<Self> {
        let mut salt = [0u8; 16];
        let rand = SystemRandom::new();
        rand.fill(&mut salt)?;

        let mut key = [0; 32];
        pbkdf2::derive(&pbkdf2::HMAC_SHA256,
                       iterations,
                       &salt,
                       password.as_bytes(),
                       &mut key);

        let keys = Keys {
            opening: OpeningKey::new(&CHACHA20_POLY1305, &key[..])?,
            sealing: SealingKey::new(&CHACHA20_POLY1305, &key[..])?,
        };

        let config = Config {
            salt: salt,
            iterations: iterations,
            keys: Some(keys),
        };

        config.save(fs);

        Ok(config)
    }

    fn save(&self, fs: &Filesystem) -> RustyPlatterResult<()> {
        let config_file = fs.open(".rusty-platter.json")?;
        Ok(())
    }
}

/// Struct that deals with a `Filesystem` implementation writing encrypted and reading decrypted.
struct EncryptedFs<'a> {
    fs: &'a Filesystem,
    config: Config,
}

impl<'a> EncryptedFs<'a> {
    fn new(fs: &'a Filesystem, config: Config) -> Self {
        EncryptedFs {
            fs: fs,
            config: config,
        }
    }

    fn encrypt_name(&self, name: &str) -> RustyPlatterResult<String> {
        assert!(name.len() >= 16);  // TODO: Add some padding
        let keys = self.config.keys.as_ref().unwrap();
        let additional_data = [];
        let mut nonce = vec![0; 12];
        let mut output: Vec<u8> = vec![];
        let mut to_encrypt: Vec<u8> = vec![];
        to_encrypt.extend_from_slice(name.as_bytes());

        // Fill nonce with random data
        let rand = SystemRandom::new();
        rand.fill(&mut nonce);
        output.extend_from_slice(&nonce);

        seal_in_place(&keys.sealing,
                      &nonce,
                      &additional_data,
                      &mut to_encrypt,
                      CHACHA20_POLY1305.tag_len())?;
        output.extend_from_slice(&to_encrypt);
        Ok(encode(&output))
    }

    fn mkdir() {
        unimplemented!()
    }
}


#[cfg(test)]
mod tests {
    extern crate ring;
    extern crate tempdir;

    use ring::aead::*;
    use ring::pbkdf2::*;
    use ring::rand::SystemRandom;

    use self::tempdir::TempDir;

    use super::*;
    use fs::local::LocalFileSystem;

    const PASSWORD: &'static str = "password";
    const ITERATIONS: u32 = 100;

    #[test]
    fn test_encrypt_name() {
        let temp = TempDir::new("test_mkdir").unwrap();
        let path = temp.path();
        let fs = LocalFileSystem::new(path.to_str().unwrap());

        let config = Config::new(PASSWORD, ITERATIONS, &fs).unwrap();
        let encrypted = EncryptedFs::new(&fs, config);
        let encrypted_name = encrypted.encrypt_name("abc").unwrap();
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
