use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use nodera_core::{
    is_windows_protocol_registered, register_windows_protocol, unregister_windows_protocol,
    NoderaUri, Vault, VaultEntry, VaultService,
};
use nodera_index::VaultIndex;
use nodera_markdown::{parse_document, LinkGraph};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let cmd = args[1].to_ascii_lowercase();
    match cmd.as_str() {
        "search" => handle_search(&args[2..]),
        "index" => handle_index(&args[2..]),
        "stats" => handle_stats(&args[2..]),
        "create" => handle_create(&args[2..]),
        "uri" => handle_uri(&args[2..]).await,
        "register-protocol" => handle_register_protocol(),
        "unregister-protocol" => handle_unregister_protocol(),
        "help" | "--help" | "-h" => print_usage(),
        other => {
            eprintln!("Unknown command: '{other}'");
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!(
        r#"Nodera Headless CLI — Knowledge Workspace Engine

USAGE:
    nodera <COMMAND> [OPTIONS]

COMMANDS:
    search <vault_path> "<query>" [--limit <n>]
        Search notes in vault with full-text BM25 ranking and snippets.

    index <vault_path>
        Rebuild SQLite and Tantivy search indices headlessly.

    stats <vault_path>
        Display detailed vault metrics (notes, words, links, orphans, tasks, tags).

    create <vault_path> "<title>" [--content "text"] [--tags "tag1,tag2"]
        Create a new Markdown note in the vault with atomic tempfile safety.

    uri "<nodera://...>"
        Dispatch a deep link to a running Nodera Desktop or execute headlessly.

    register-protocol
        Register 'nodera://' protocol handler in Windows registry (HKCU).

    unregister-protocol
        Remove 'nodera://' protocol handler from Windows registry.

    help
        Print this help message.
"#
    );
}

fn handle_search(args: &[String]) {
    if args.len() < 2 {
        eprintln!("Usage: nodera search <vault_path> \"<query>\" [--limit <n>]");
        std::process::exit(1);
    }

    let vault_path = Path::new(&args[0]);
    let query = &args[1];
    let limit = args
        .iter()
        .position(|a| a == "--limit")
        .and_then(|i| args.get(i + 1))
        .and_then(|l| l.parse::<usize>().ok())
        .unwrap_or(10);

    let idx = match VaultIndex::open(vault_path) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Failed to open index at {}: {e}", vault_path.display());
            std::process::exit(1);
        }
    };

    let start = Instant::now();
    match idx.search(query, limit) {
        Ok(results) => {
            let elapsed = start.elapsed();
            println!(
                "Found {} results for '{}' ({:?}):\n",
                results.len(),
                query,
                elapsed
            );
            for (i, res) in results.iter().enumerate() {
                println!(
                    "{}. {} ({}) [score: {:.2}]",
                    i + 1,
                    res.title,
                    res.path,
                    res.score
                );
                if !res.snippet.trim().is_empty() {
                    println!("   Snippet: {}", res.snippet.trim());
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("Search error: {e}");
            std::process::exit(1);
        }
    }
}

fn handle_index(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: nodera index <vault_path>");
        std::process::exit(1);
    }

    let vault_path = Path::new(&args[0]);
    let vault = match Vault::open(vault_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to open vault at {}: {e}", vault_path.display());
            std::process::exit(1);
        }
    };

    let service = VaultService::new(vault);
    let mut index = match VaultIndex::open(vault_path) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("Failed to open vault index: {e}");
            std::process::exit(1);
        }
    };

    println!("Rebuilding index for vault at {}...", vault_path.display());
    let start = Instant::now();
    match index.rebuild_with_progress(
        &service,
        |p| {
            print!("\rIndexing: {:.0}% ({:?})    ", p.percentage(), p.phase);
            let _ = std::io::stdout().flush();
        },
        None,
    ) {
        Ok(count) => {
            let elapsed = start.elapsed();
            println!();
            let nps = if elapsed.as_secs_f64() > 0.0 {
                count as f64 / elapsed.as_secs_f64()
            } else {
                0.0
            };
            println!(
                "Successfully indexed {} notes in {:?} ({:.1} notes/sec).",
                count, elapsed, nps
            );
        }
        Err(e) => {
            eprintln!("\nIndex rebuild failed: {e}");
            std::process::exit(1);
        }
    }
}

fn handle_stats(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: nodera stats <vault_path>");
        std::process::exit(1);
    }

    let vault_path = Path::new(&args[0]);
    let vault = match Vault::open(vault_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to open vault at {}: {e}", vault_path.display());
            std::process::exit(1);
        }
    };

    let vault_name = vault.config().name.clone();
    let service = VaultService::new(vault);

    let entries = match service.list_entries() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to list vault entries: {e}");
            std::process::exit(1);
        }
    };

    let mut note_count = 0;
    let mut folder_count = 0;
    let mut total_bytes = 0u64;
    let mut total_words = 0usize;
    let mut total_tasks = 0usize;
    let mut done_tasks = 0usize;
    let mut note_paths = Vec::new();
    let mut note_contents = HashMap::new();
    let mut all_tags = std::collections::HashSet::new();

    for entry in &entries {
        match entry {
            VaultEntry::Folder { .. } => folder_count += 1,
            VaultEntry::Note(summary) => {
                note_count += 1;
                total_bytes += summary.size_bytes;
                note_paths.push(summary.relative_path.clone());

                if let Ok(note) = service.read_note(&summary.relative_path) {
                    let words = note.content.split_whitespace().count();
                    total_words += words;

                    if let Ok(parsed) = parse_document(&note.content) {
                        for t in parsed.tasks {
                            total_tasks += 1;
                            if t.checked {
                                done_tasks += 1;
                            }
                        }
                        for tag in parsed.tags {
                            all_tags.insert(tag);
                        }
                    }
                    note_contents.insert(summary.relative_path.clone(), note.content);
                }
            }
        }
    }

    // Build link graph and run audit
    let mut note_links = Vec::new();
    for (p, content) in &note_contents {
        if let Ok(parsed) = parse_document(content) {
            note_links.push((p.clone(), parsed.wikilinks));
        }
    }

    let graph = LinkGraph::build(&note_paths, note_links);
    let audit = graph.audit_vault_links(&note_paths, &note_contents);

    println!("============================================================");
    println!(" Nodera Vault Statistics: {}", vault_name);
    println!("============================================================");
    println!(" Path:            {}", vault_path.display());
    println!(" Total Notes:     {}", note_count);
    println!(" Folders:         {}", folder_count);
    println!(" Total Storage:   {:.2} KB", total_bytes as f64 / 1024.0);
    println!(" Total Words:     {}", total_words);
    println!(" Unique Tags:     {}", all_tags.len());
    println!(
        " Tasks:           {} ({} open, {} done)",
        total_tasks,
        total_tasks - done_tasks,
        done_tasks
    );
    println!(" Total Links:     {}", audit.total_links);
    println!(" Broken Links:    {}", audit.broken_links.len());
    println!(" Orphan Notes:    {}", audit.orphan_notes.len());
    println!(
        " Windows URI:     {}",
        if is_windows_protocol_registered() {
            "Registered (nodera://)"
        } else {
            "Not registered"
        }
    );
    println!("============================================================");
}

fn handle_create(args: &[String]) {
    if args.len() < 2 {
        eprintln!(
            "Usage: nodera create <vault_path> \"<title>\" [--content \"text\"] [--tags \"tag1,tag2\"]"
        );
        std::process::exit(1);
    }

    let vault_path = Path::new(&args[0]);
    let title = &args[1];

    let mut content: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();

    let mut i = 2;
    while i < args.len() {
        if args[i] == "--content" && i + 1 < args.len() {
            content = Some(args[i + 1].clone());
            i += 1;
        } else if args[i] == "--tags" && i + 1 < args.len() {
            tags = args[i + 1]
                .split(',')
                .map(|s| s.trim().trim_start_matches('#').to_string())
                .filter(|s| !s.is_empty())
                .collect();
            i += 1;
        }
        i += 1;
    }

    let vault = match Vault::open(vault_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to open vault at {}: {e}", vault_path.display());
            std::process::exit(1);
        }
    };

    let service = VaultService::new(vault);

    let mut body = String::new();
    if !tags.is_empty() {
        body.push_str("---\ntags:\n");
        for t in &tags {
            body.push_str(&format!("  - {}\n", t));
        }
        body.push_str("---\n\n");
    }
    if let Some(c) = content {
        body.push_str(&c);
    }

    let folder = Path::new(title)
        .parent()
        .and_then(|p| p.to_str())
        .filter(|f| !f.is_empty());
    let title_stem = Path::new(title)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(title);

    match service.create_note(folder, title_stem, Some(&body)) {
        Ok(note) => {
            println!(
                "Successfully created note: {}",
                note.relative_path.display()
            );
            // Update search index if available
            if let Ok(mut idx) = VaultIndex::open(vault_path) {
                if let Ok(parsed) = parse_document(&note.content) {
                    let _ = idx.index_note(&note, &parsed);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to create note: {e}");
            std::process::exit(1);
        }
    }
}

async fn handle_uri(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: nodera uri \"<nodera://...>\"");
        std::process::exit(1);
    }

    let uri_str = &args[0];
    let parsed_uri = match NoderaUri::parse(uri_str) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Invalid URI: {e}");
            std::process::exit(1);
        }
    };

    // 1. Try sending to running instance via loopback TCP
    let port = 27123;
    if send_uri_sync_client(uri_str, port) {
        println!("URI successfully dispatched to active Nodera Desktop instance.");
        return;
    }

    // 2. Otherwise execute headlessly
    println!("No running Nodera instance detected. Executing URI action headlessly...");
    match parsed_uri {
        NoderaUri::Open { vault, note, .. } => {
            println!("Action: Open Note '{}' (Vault: {:?})", note, vault);
        }
        NoderaUri::New {
            vault,
            title,
            content,
            tags,
        } => {
            println!("Action: Create Note '{}' (Vault: {:?})", title, vault);
            if let Some(v_path) = vault {
                let v_ref = Path::new(&v_path);
                if v_ref.is_dir() {
                    let v = Vault::open(v_ref).expect("Open vault");
                    let s = VaultService::new(v);
                    let mut b = String::new();
                    if !tags.is_empty() {
                        b.push_str("---\ntags:\n");
                        for t in &tags {
                            b.push_str(&format!("  - {}\n", t));
                        }
                        b.push_str("---\n\n");
                    }
                    if let Some(c) = content {
                        b.push_str(&c);
                    }
                    let res = s.create_note(None, &title, Some(&b));
                    match res {
                        Ok(n) => println!("Created '{}'", n.relative_path.display()),
                        Err(e) => eprintln!("Failed: {e}"),
                    }
                }
            }
        }
        NoderaUri::Search { vault, query } => {
            println!("Action: Search '{}' (Vault: {:?})", query, vault);
            if let Some(v_path) = vault {
                let v_ref = Path::new(&v_path);
                if let Ok(idx) = VaultIndex::open(v_ref) {
                    if let Ok(res) = idx.search(&query, 5) {
                        for r in res {
                            println!("- {} ({})", r.title, r.path);
                        }
                    }
                }
            }
        }
        NoderaUri::Daily { vault } => {
            println!("Action: Daily Note (Vault: {:?})", vault);
        }
    }
}

fn send_uri_sync_client(uri_str: &str, port: u16) -> bool {
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    let addr_str = format!("127.0.0.1:{}", port);
    let addr: SocketAddr = match addr_str.parse() {
        Ok(a) => a,
        Err(_) => return false,
    };

    let mut stream = match TcpStream::connect_timeout(&addr, Duration::from_millis(150)) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
    let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));

    let payload = serde_json::json!({ "uri": uri_str }).to_string();
    let request = format!(
        "POST /uri HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        port,
        payload.len(),
        payload
    );

    if stream.write_all(request.as_bytes()).is_ok() {
        let mut resp = [0u8; 1024];
        if let Ok(n) = stream.read(&mut resp) {
            let resp_str = String::from_utf8_lossy(&resp[..n]);
            return resp_str.contains("200 OK");
        }
    }
    false
}

fn handle_register_protocol() {
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("nodera.exe"));
    match register_windows_protocol(&current_exe) {
        Ok(_) => {
            println!(
                "Successfully registered 'nodera://' protocol handler to: {}",
                current_exe.display()
            );
        }
        Err(e) => {
            eprintln!("Failed to register protocol: {e}");
            std::process::exit(1);
        }
    }
}

fn handle_unregister_protocol() {
    match unregister_windows_protocol() {
        Ok(_) => {
            println!(
                "Successfully unregistered 'nodera://' protocol handler from Windows registry."
            );
        }
        Err(e) => {
            eprintln!("Failed to unregister protocol: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cli_vault_stats_and_create() {
        let tmp = tempdir().unwrap();
        let vault_path = tmp.path().join("CliVault");
        let vault = Vault::create(&vault_path, Some("CLI Test Vault".to_string())).unwrap();
        let service = VaultService::new(vault);

        // Create test notes
        service
            .create_note(
                None,
                "Note_A",
                Some("# Note A\nLinks to [[Note_B]]. #research\n- [ ] Task 1\n- [x] Task 2"),
            )
            .unwrap();

        service
            .create_note(
                None,
                "Note_B",
                Some("# Note B\nDestination note.\n- [ ] Task 3"),
            )
            .unwrap();

        // Check stats computation
        let entries = service.list_entries().unwrap();
        let notes: Vec<_> = entries
            .iter()
            .filter(|e| matches!(e, VaultEntry::Note(_)))
            .collect();
        assert_eq!(notes.len(), 2);

        // Test CLI create
        handle_create(&[
            vault_path.to_string_lossy().to_string(),
            "CLI_Note_C".to_string(),
            "--content".to_string(),
            "Created from automated test".to_string(),
            "--tags".to_string(),
            "automation,cli".to_string(),
        ]);

        let created_file = vault_path.join("Notes").join("CLI_Note_C.md");
        assert!(created_file.exists());
        let text = std::fs::read_to_string(&created_file).unwrap();
        assert!(text.contains("automation"));
        assert!(text.contains("Created from automated test"));

        // Test CLI index
        handle_index(&[vault_path.to_string_lossy().to_string()]);

        // Test CLI search
        let idx = VaultIndex::open(&vault_path).unwrap();
        let results = idx.search("automation", 5).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "CLI_Note_C");
    }

    #[tokio::test]
    async fn test_cli_uri_dispatch() {
        let uri = "nodera://open?vault=TestVault&note=Daily/2026-09-19.md";
        let parsed = NoderaUri::parse(uri).unwrap();
        match parsed {
            NoderaUri::Open { vault, note, .. } => {
                assert_eq!(vault.as_deref(), Some("TestVault"));
                assert_eq!(note, "Daily/2026-09-19.md");
            }
            _ => panic!("Expected Open variant"),
        }
    }
}
