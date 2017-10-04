use data_encoding;
use ring;
use std::io;

error_chain! {
    foreign_links {
        IoError(io::Error);
        CryptoError(ring::error::Unspecified);
        Base32Error(data_encoding::DecodeError);
    }

    errors {
        InvalidEncodedName {
            description("Invalid encoded name")
            display("Invalid encoded name")
        }
        InvalidPathName(path: String) {
            description("Invalid path name")
            display("Invalid path name {}", path)
        }
        IterationsNumberTooSmall(iterations: u32) {
            description("Iterations number too small")
            display("{} iterations is too small", iterations)
        }
    }
}
