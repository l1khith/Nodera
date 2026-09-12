use std::fs;
use std::path::PathBuf;

fn branding_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("assets")
        .join("branding")
}

#[test]
fn test_canonical_svg_assets_exist() {
    let dir = branding_dir();
    let logo_svg = dir.join("nodera-logo.svg");
    let mono_svg = dir.join("nodera-logo-mono.svg");

    assert!(
        logo_svg.exists(),
        "nodera-logo.svg must exist in assets/branding"
    );
    assert!(
        mono_svg.exists(),
        "nodera-logo-mono.svg must exist in assets/branding"
    );

    let logo_content = fs::read_to_string(&logo_svg).expect("Read logo SVG");
    assert!(
        logo_content.contains("<svg") && logo_content.contains("</svg>"),
        "Must be valid SVG"
    );
    assert!(
        logo_content.contains("viewBox=\"0 0 512 512\""),
        "Must have 512x512 viewBox"
    );

    let mono_content = fs::read_to_string(&mono_svg).expect("Read mono SVG");
    assert!(
        mono_content.contains("<svg") && mono_content.contains("</svg>"),
        "Must be valid SVG"
    );
}

#[test]
fn test_multi_resolution_png_assets_exist() {
    let dir = branding_dir();
    let required_sizes = [16, 24, 32, 48, 64, 128, 256];

    for size in required_sizes {
        let png = dir.join(format!("nodera-icon-{size}.png"));
        assert!(png.exists(), "nodera-icon-{size}.png must exist");
        let meta = fs::metadata(&png).expect("PNG metadata");
        assert!(meta.len() > 100, "nodera-icon-{size}.png must not be empty");
    }

    let default_png = dir.join("nodera-icon.png");
    assert!(default_png.exists(), "Default nodera-icon.png must exist");
}

#[test]
fn test_windows_multi_res_ico_valid() {
    let ico_path = branding_dir().join("nodera.ico");
    assert!(
        ico_path.exists(),
        "nodera.ico must exist in assets/branding"
    );

    let bytes = fs::read(&ico_path).expect("Read ICO file");
    assert!(bytes.len() > 1000, "ICO file must be non-trivial");

    // ICO format header: 0x0000 reserved, 0x0001 icon type, count (u16 LE)
    assert_eq!(&bytes[0..2], &[0x00, 0x00], "Reserved bytes must be 0");
    assert_eq!(&bytes[2..4], &[0x01, 0x00], "Type must be 1 (icon)");
    let count = u16::from_le_bytes([bytes[4], bytes[5]]);
    assert!(
        count >= 7,
        "ICO must contain at least 7 resolutions (16, 24, 32, 48, 64, 128, 256), found {count}"
    );
}

#[test]
fn test_runtime_raw_rgba_binary_valid() {
    let bin_path = branding_dir().join("nodera-icon-256.bin");
    assert!(bin_path.exists(), "nodera-icon-256.bin must exist");

    let meta = fs::metadata(&bin_path).expect("BIN metadata");
    let expected_len = 256 * 256 * 4; // 256x256 RGBA
    assert_eq!(
        meta.len(),
        expected_len as u64,
        "nodera-icon-256.bin must be exactly 262,144 bytes for 256x256 RGBA"
    );
}

#[test]
fn test_macos_icns_exists() {
    let icns_path = branding_dir().join("nodera.icns");
    assert!(
        icns_path.exists(),
        "nodera.icns must exist in assets/branding"
    );
    let meta = fs::metadata(&icns_path).expect("ICNS metadata");
    assert!(meta.len() > 1000, "ICNS file must be non-trivial");
}

#[test]
fn test_dioxus_bundle_config_references_branding() {
    let repo_root = branding_dir()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let config_path = repo_root.join("Dioxus.toml");
    assert!(
        config_path.exists(),
        "Dioxus.toml must exist in repository root"
    );

    let content = fs::read_to_string(&config_path).expect("Read Dioxus.toml");
    assert!(
        content.contains("assets/branding/nodera.ico"),
        "Must configure Windows .ico"
    );
    assert!(
        content.contains("assets/branding/nodera.icns"),
        "Must configure macOS .icns"
    );
    assert!(
        content.contains("assets/branding/nodera-icon-256.png"),
        "Must configure PNG icon"
    );
    assert!(
        content.contains("name = \"nodera\""),
        "Application name must be nodera"
    );
}
