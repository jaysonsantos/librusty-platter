use fs::Filesystem;
use result::{RustyPlatterResult, Error};

use ring::aead::{SealingKey, OpeningKey, CHACHA20_POLY1305};
use ring::pbkdf2;
use ring::rand::SystemRandom;

const MINIMUM_ITERATIONS: u32 = 10_000;

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
        if iterations < MINIMUM_ITERATIONS {
            return Err(Error::IterationsNumberTooSmall);
        }
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
        // let config_file = fs.open(".rusty-platter.json")?;
        Ok(())
    }
}
