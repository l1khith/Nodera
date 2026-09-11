//! Desktop entry point for Nodera.

fn main() {
    println!("Nodera Desktop");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_desktop_smoke() {
        let app_name = "Nodera";
        assert_eq!(app_name, "Nodera");
    }
}
