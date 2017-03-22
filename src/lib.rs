pub mod fs;
pub mod result;

/// # RustyPlatter
pub struct RustyPlatter {
    // fs: Filesystem
}

impl RustyPlatter {
    fn new() -> Self {
        let rusty_platter = RustyPlatter {};
        rusty_platter
    }
}

#[cfg(test)]
mod tests {
    use ::RustyPlatter;
    #[test]
    fn test_nothing() {
        let a = RustyPlatter::new();
    }
}
