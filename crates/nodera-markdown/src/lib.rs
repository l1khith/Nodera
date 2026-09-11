//! Markdown processing and knowledge extraction crate for Nodera.

pub fn placeholder() -> &'static str {
    "nodera-markdown"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert_eq!(placeholder(), "nodera-markdown");
    }
}
