extern crate base64;
extern crate ring;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

pub mod fs;
pub mod result;

// /// # RustyPlatter
// pub struct RustyPlatter<'a> {
//     fs: &'a fs::Filesystem,
// }

// impl<'a> RustyPlatter<'a> {
//     fn new(fs: &'a fs::Filesystem) -> Self {
//         let rusty_platter = RustyPlatter { fs: fs };
//         rusty_platter
//     }
// }

// #[cfg(test)]
// mod tests {
//     use ::RustyPlatter;
//     use ::fs::local::LocalFileSystem;

//     #[test]
//     fn test_nothing() {
//         let fs = LocalFileSystem::new(".");
//         let a = RustyPlatter::new(&fs);
//     }
// }
