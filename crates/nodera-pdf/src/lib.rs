//! PDF extraction and conversion crate for Nodera.

pub fn placeholder() -> &'static str {
    "nodera-pdf"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert_eq!(placeholder(), "nodera-pdf");
    }
}
