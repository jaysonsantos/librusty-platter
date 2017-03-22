pub struct RustyPlatter;

impl RustyPlatter {
    fn new() -> Self {
        let rusty_platter = RustyPlatter {};
        rusty_platter
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_nothing() {
        let a = RustyPlatter::new();
    }
}
