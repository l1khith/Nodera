//! Indexing and search derivation crate for Nodera.

pub fn placeholder() -> &'static str {
    "nodera-index"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert_eq!(placeholder(), "nodera-index");
    }
}
