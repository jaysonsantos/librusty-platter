/// # RustyPlatter
pub struct RustyPlatter;

pub mod fs;
pub mod result;

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
