//! Version-locked composition and integration runtime for native Quire V1.

/// Placeholder entry point.
pub fn hello() -> &'static str {
    "hello from quire_integration"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_returns_greeting() {
        assert!(hello().contains("quire_integration"));
    }
}
