use fs::Filesystem;
use result::{RustyPlatterResult, Error};

use ring::aead::{SealingKey, OpeningKey, CHACHA20_POLY1305, seal_in_place, open_in_place};
use ring::pbkdf2;
use ring::rand::{SystemRandom, SecureRandom};

use serde_json;

pub struct Keys {
    pub opening: OpeningKey,
    pub sealing: SealingKey,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    salt: [u8; 16],

    iterations: u32,

    #[serde(skip_serializing, skip_deserializing)]
    pub keys: Option<Keys>,
}

impl Config {
    #![allow(dead_code)]
    pub fn new(password: &str, iterations: u32, fs: &Filesystem) -> RustyPlatterResult<Self> {
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

        config.save(fs)?;

        Ok(config)
    }

    fn save(&self, fs: &Filesystem) -> RustyPlatterResult<()> {
        let config_file = fs.open(".rusty-platter.json")?;
        Ok(())
    }
}
