use crate::error::{NoderaError, Result, UriError};
use std::collections::HashMap;
use std::path::Path;

/// Custom URI actions supported by Nodera.
///
/// Supported schemes:
/// - `nodera://open?vault=<vault>&note=<relative_path>&line=<line>`
/// - `nodera://new?vault=<vault>&title=<title>&content=<content>&tags=<tag1,tag2>`
/// - `nodera://search?vault=<vault>&query=<query>`
/// - `nodera://daily?vault=<vault>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoderaUri {
    /// Open an existing note in a specified (or currently open) vault.
    Open {
        vault: Option<String>,
        note: String,
        line: Option<usize>,
    },
    /// Create a new note with optional content and tags.
    New {
        vault: Option<String>,
        title: String,
        content: Option<String>,
        tags: Vec<String>,
    },
    /// Trigger a full-text or title search in the vault.
    Search {
        vault: Option<String>,
        query: String,
    },
    /// Open or create today's daily note.
    Daily { vault: Option<String> },
}

impl NoderaUri {
    /// Parses a URI string such as `nodera://open?note=Daily/2026-09-19.md`.
    pub fn parse(uri_str: &str) -> Result<Self> {
        let trimmed = uri_str.trim();

        // Check scheme
        let prefix = "nodera://";
        if !trimmed.to_ascii_lowercase().starts_with(prefix) {
            return Err(NoderaError::Uri(UriError::InvalidScheme(format!(
                "URI must start with '{prefix}', got '{trimmed}'"
            ))));
        }

        let after_scheme = &trimmed[prefix.len()..];

        // Split action and query string
        let (action_part, query_part) = match after_scheme.split_once('?') {
            Some((action, query)) => (action.trim_end_matches('/'), query),
            None => (after_scheme.trim_end_matches('/'), ""),
        };

        let action = action_part.to_ascii_lowercase();
        let params = parse_query_string(query_part);

        let vault = params
            .get("vault")
            .cloned()
            .filter(|v| !v.trim().is_empty());

        match action.as_str() {
            "open" => {
                let note = params
                    .get("note")
                    .or_else(|| params.get("file"))
                    .or_else(|| params.get("path"))
                    .cloned()
                    .unwrap_or_default();

                if note.trim().is_empty() {
                    return Err(NoderaError::Uri(UriError::MissingParameter(
                        "Missing required 'note' parameter in nodera://open URI".to_string(),
                    )));
                }

                let line = params.get("line").and_then(|l| l.parse::<usize>().ok());

                Ok(NoderaUri::Open { vault, note, line })
            }
            "new" | "create" => {
                let title = params
                    .get("title")
                    .or_else(|| params.get("name"))
                    .cloned()
                    .unwrap_or_default();

                if title.trim().is_empty() {
                    return Err(NoderaError::Uri(UriError::MissingParameter(
                        "Missing required 'title' parameter in nodera://new URI".to_string(),
                    )));
                }

                let content = params
                    .get("content")
                    .or_else(|| params.get("body"))
                    .cloned()
                    .filter(|c| !c.is_empty());

                let tags = params
                    .get("tags")
                    .map(|t| {
                        t.split(',')
                            .map(|s| s.trim().trim_start_matches('#').to_string())
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                Ok(NoderaUri::New {
                    vault,
                    title,
                    content,
                    tags,
                })
            }
            "search" | "find" => {
                let query = params
                    .get("query")
                    .or_else(|| params.get("q"))
                    .cloned()
                    .unwrap_or_default();

                Ok(NoderaUri::Search { vault, query })
            }
            "daily" => Ok(NoderaUri::Daily { vault }),
            other => Err(NoderaError::Uri(UriError::UnsupportedAction(format!(
                "Unsupported nodera:// action '{other}'"
            )))),
        }
    }

    /// Serializes this URI into a standard `nodera://...` string.
    pub fn to_uri_string(&self) -> String {
        match self {
            NoderaUri::Open { vault, note, line } => {
                let mut query = Vec::new();
                if let Some(v) = vault {
                    query.push(format!("vault={}", percent_encode(v)));
                }
                query.push(format!("note={}", percent_encode(note)));
                if let Some(l) = line {
                    query.push(format!("line={}", l));
                }
                format!("nodera://open?{}", query.join("&"))
            }
            NoderaUri::New {
                vault,
                title,
                content,
                tags,
            } => {
                let mut query = Vec::new();
                if let Some(v) = vault {
                    query.push(format!("vault={}", percent_encode(v)));
                }
                query.push(format!("title={}", percent_encode(title)));
                if let Some(c) = content {
                    query.push(format!("content={}", percent_encode(c)));
                }
                if !tags.is_empty() {
                    query.push(format!("tags={}", percent_encode(&tags.join(","))));
                }
                format!("nodera://new?{}", query.join("&"))
            }
            NoderaUri::Search { vault, query } => {
                let mut q = Vec::new();
                if let Some(v) = vault {
                    q.push(format!("vault={}", percent_encode(v)));
                }
                q.push(format!("query={}", percent_encode(query)));
                format!("nodera://search?{}", q.join("&"))
            }
            NoderaUri::Daily { vault } => {
                if let Some(v) = vault {
                    format!("nodera://daily?vault={}", percent_encode(v))
                } else {
                    "nodera://daily".to_string()
                }
            }
        }
    }
}

/// Parses a URL query string (`key=value&key2=value2`) into a key-value map with percent decoding.
pub fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if query.trim().is_empty() {
        return map;
    }

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (raw_key, raw_val) = match pair.split_once('=') {
            Some((k, v)) => (k, v),
            None => (pair, ""),
        };
        let key = percent_decode(raw_key).trim().to_ascii_lowercase();
        let val = percent_decode(raw_val);
        if !key.is_empty() {
            map.insert(key, val);
        }
    }
    map
}

/// Decodes standard percent-encoded strings (`%20` -> `' '`, `%0A` -> `'\n'`).
pub fn percent_decode(input: &str) -> String {
    let mut bytes = Vec::with_capacity(input.len());
    let mut chars = input.bytes();

    while let Some(b) = chars.next() {
        if b == b'%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(c1), Some(c2)) = (h1, h2) {
                if let (Some(v1), Some(v2)) = (hex_digit(c1), hex_digit(c2)) {
                    bytes.push((v1 << 4) | v2);
                    continue;
                }
                bytes.push(b'%');
                bytes.push(c1);
                bytes.push(c2);
            } else {
                bytes.push(b'%');
                if let Some(c1) = h1 {
                    bytes.push(c1);
                }
            }
        } else if b == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(b);
        }
    }

    String::from_utf8_lossy(&bytes).to_string()
}

/// Encodes special characters in query string values according to RFC 3986 unreserved characters.
pub fn percent_encode(input: &str) -> String {
    let mut output = String::with_capacity(input.len() * 2);
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                output.push(b as char);
            }
            _ => {
                output.push_str(&format!("%{:02X}", b));
            }
        }
    }
    output
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Windows Protocol Registration Engine
// ---------------------------------------------------------------------------

/// Registers the `nodera://` protocol handler in the current user's registry:
/// `HKCU\Software\Classes\nodera` -> points to `<exe_path> --uri "%1"`.
///
/// This does not require administrator privileges because it is in `HKCU`.
pub fn register_windows_protocol(exe_path: &Path) -> Result<()> {
    if !cfg!(target_os = "windows") {
        return Ok(());
    }

    let exe_str = exe_path.to_string_lossy();
    let command_val = format!("\"{}\" --uri \"%1\"", exe_str);

    // 1. Root key HKCU\Software\Classes\nodera
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\nodera",
        "/ve",
        "/d",
        "URL:Nodera Protocol",
        "/f",
    ])?;

    // 2. URL Protocol value
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\nodera",
        "/v",
        "URL Protocol",
        "/t",
        "REG_SZ",
        "/d",
        "",
        "/f",
    ])?;

    // 3. Command subkey
    run_reg(&[
        "add",
        r"HKCU\Software\Classes\nodera\shell\open\command",
        "/ve",
        "/d",
        &command_val,
        "/f",
    ])?;

    Ok(())
}

/// Unregisters the `nodera://` protocol handler from `HKCU\Software\Classes\nodera`.
pub fn unregister_windows_protocol() -> Result<()> {
    if !cfg!(target_os = "windows") {
        return Ok(());
    }

    run_reg(&["delete", r"HKCU\Software\Classes\nodera", "/f"])?;
    Ok(())
}

/// Checks whether the `nodera://` protocol handler is currently registered in `HKCU`.
pub fn is_windows_protocol_registered() -> bool {
    if !cfg!(target_os = "windows") {
        return false;
    }

    std::process::Command::new("reg")
        .args(["query", r"HKCU\Software\Classes\nodera"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_reg(args: &[&str]) -> Result<()> {
    let output = std::process::Command::new("reg")
        .args(args)
        .output()
        .map_err(|e| {
            NoderaError::Uri(UriError::RegistrationError(format!(
                "Failed to execute reg command: {e}"
            )))
        })?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(NoderaError::Uri(UriError::RegistrationError(format!(
            "reg command failed: {err}"
        ))));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_encode_decode() {
        let text = "Hello World & Welcome! / Daily: 2026-09-19";
        let encoded = percent_encode(text);
        assert!(!encoded.contains(' '));
        let decoded = percent_decode(&encoded);
        assert_eq!(decoded, text);

        // Test plus sign decode
        assert_eq!(percent_decode("hello+world"), "hello world");
    }

    #[test]
    fn test_parse_open_uri() {
        let uri =
            NoderaUri::parse("nodera://open?vault=MyVault&note=Daily%2F2026-09-19.md&line=42")
                .unwrap();
        assert_eq!(
            uri,
            NoderaUri::Open {
                vault: Some("MyVault".to_string()),
                note: "Daily/2026-09-19.md".to_string(),
                line: Some(42),
            }
        );

        let roundtrip = NoderaUri::parse(&uri.to_uri_string()).unwrap();
        assert_eq!(roundtrip, uri);
    }

    #[test]
    fn test_parse_new_note_uri() {
        let uri = NoderaUri::parse("nodera://new?title=Project%20Alpha&content=%23%20Header%0ABody%20text&tags=ai,research").unwrap();
        assert_eq!(
            uri,
            NoderaUri::New {
                vault: None,
                title: "Project Alpha".to_string(),
                content: Some("# Header\nBody text".to_string()),
                tags: vec!["ai".to_string(), "research".to_string()],
            }
        );

        let roundtrip = NoderaUri::parse(&uri.to_uri_string()).unwrap();
        assert_eq!(roundtrip, uri);
    }

    #[test]
    fn test_parse_search_uri() {
        let uri = NoderaUri::parse("nodera://search?q=neural%20networks").unwrap();
        assert_eq!(
            uri,
            NoderaUri::Search {
                vault: None,
                query: "neural networks".to_string(),
            }
        );

        let roundtrip = NoderaUri::parse(&uri.to_uri_string()).unwrap();
        assert_eq!(roundtrip, uri);
    }

    #[test]
    fn test_parse_daily_uri() {
        let uri = NoderaUri::parse("nodera://daily?vault=Research").unwrap();
        assert_eq!(
            uri,
            NoderaUri::Daily {
                vault: Some("Research".to_string()),
            }
        );

        let roundtrip = NoderaUri::parse(&uri.to_uri_string()).unwrap();
        assert_eq!(roundtrip, uri);
    }

    #[test]
    fn test_invalid_uris() {
        assert!(NoderaUri::parse("https://open?note=Test").is_err());
        assert!(NoderaUri::parse("nodera://open").is_err()); // missing note
        assert!(NoderaUri::parse("nodera://new").is_err()); // missing title
        assert!(NoderaUri::parse("nodera://unknown_action").is_err());
    }
}
