extern crate base64;
extern crate ring;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

pub mod config;
pub mod fs;
pub mod result;

pub use config::Config;
pub use fs::encrypted::EncryptedFs;
pub use result::{Error, RustyPlatterResult};
