use fs::Filesystem;
use result::{RustyPlatterResult, Error};

use ring::aead::{SealingKey, OpeningKey, CHACHA20_POLY1305};
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};

use serde_json;

const MINIMUM_ITERATIONS: u32 = 10_000;
const CONFIG_PATH: &'static str = ".rusty-platter.json";

pub struct Keys {
    pub opening: OpeningKey,
    pub sealing: SealingKey,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    salt: [u8; 16],

    iterations: u32,

    /// Keys are stored as option to make it optional when serializing but they will always
    /// be there, so it is safe (?) to call unwrap
    #[serde(skip_serializing, skip_deserializing)]
    keys: Option<Keys>,
}

impl Config {
    #![allow(dead_code)]
    pub fn new(password: &str, iterations: u32, fs: &Filesystem) -> RustyPlatterResult<Self> {
        let rand = Box::new(SystemRandom::new());
        Self::new_with_custom_random(password, iterations, fs, rand)
    }

    pub fn sealing_key(&self) -> &SealingKey {
        &self.keys.as_ref().unwrap().sealing
    }

    pub fn openning_key(&self) -> &OpeningKey {
        &self.keys.as_ref().unwrap().opening
    }

    /// Create a new config with a custom random, mainly used with tests but could also be
    /// used to get some data from random.org for example
    pub fn new_with_custom_random(password: &str,
                                  iterations: u32,
                                  fs: &Filesystem,
                                  rand: Box<SecureRandom>)
                                  -> RustyPlatterResult<Self> {
        if iterations < MINIMUM_ITERATIONS {
            return Err(Error::IterationsNumberTooSmall);
        }

        let mut salt = [0u8; 16];
        rand.fill(&mut salt)?;

        let mut key = [0; 32];
        pbkdf2::derive(&pbkdf2::HMAC_SHA256,
                       iterations,
                       &salt,
                       password.as_bytes(),
                       &mut key);

        let keys = Keys {
            opening: OpeningKey::new(&CHACHA20_POLY1305, &key)?,
            sealing: SealingKey::new(&CHACHA20_POLY1305, &key)?,
        };

        let config = Config {
            salt: salt,
            iterations: iterations,
            keys: Some(keys),
        };

        config.save(fs)?;

        Ok(config)
    }

    /// Save current config to FS_ROOT/.rusty-platter.json
    pub fn save(&self, fs: &Filesystem) -> RustyPlatterResult<()> {
        let mut config_file = fs.open(CONFIG_PATH)?;
        serde_json::to_writer(&mut config_file, &self).unwrap();
        Ok(())
    }
}
