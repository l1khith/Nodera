fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../../assets/branding/nodera.ico");
        res.set("ProductName", "Nodera");
        res.set("FileDescription", "Nodera — Knowledge Workspace");
        res.set("LegalCopyright", "Nodera Contributors");
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=Failed to compile Windows resources: {}", e);
        }
    }
}
