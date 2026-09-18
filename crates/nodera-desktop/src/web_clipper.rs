use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, info, warn};

pub const DEFAULT_WEB_CLIPPER_PORT: u16 = 27123;

/// Payload received from browser extension or web clipper bookmarklet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebClipPayload {
    pub url: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub selected_text: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub author: Option<String>,
}

/// Handle to the running web clipper background HTTP server.
pub struct WebClipperHandle {
    pub port: u16,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl WebClipperHandle {
    /// Stops the web clipper server gracefully.
    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for WebClipperHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Starts the local Web Clipper HTTP server on `127.0.0.1:port`.
///
/// Sends created note paths through `notify_tx` when new clips are received.
pub async fn start_web_clipper(
    vault_root: PathBuf,
    vault_name: String,
    port: u16,
    notify_tx: Option<mpsc::UnboundedSender<PathBuf>>,
) -> Result<WebClipperHandle, std::io::Error> {
    let addr = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    let bound_port = listener.local_addr()?.port();
    info!(
        "Web Clipper HTTP server listening on http://127.0.0.1:{}",
        bound_port
    );

    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
    let vault_root = Arc::new(vault_root);
    let vault_name = Arc::new(vault_name);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    info!("Web Clipper server shutting down");
                    break;
                }
                accept_res = listener.accept() => {
                    match accept_res {
                        Ok((mut socket, _)) => {
                            let root = Arc::clone(&vault_root);
                            let vname = Arc::clone(&vault_name);
                            let n_tx = notify_tx.clone();

                            tokio::spawn(async move {
                                if let Err(e) = handle_http_connection(&mut socket, &root, &vname, n_tx).await {
                                    warn!("Error handling Web Clipper connection: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Web Clipper accept error: {}", e);
                            tokio::time::sleep(Duration::from_millis(50)).await;
                        }
                    }
                }
            }
        }
    });

    Ok(WebClipperHandle {
        port: bound_port,
        shutdown_tx: Some(shutdown_tx),
    })
}

async fn handle_http_connection(
    socket: &mut tokio::net::TcpStream,
    vault_root: &Path,
    vault_name: &str,
    notify_tx: Option<mpsc::UnboundedSender<PathBuf>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buf = vec![0u8; 8192];
    let n = socket.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }

    let request_str = String::from_utf8_lossy(&buf[..n]);
    let mut lines = request_str.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return Ok(()),
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        send_response(
            socket,
            400,
            "Bad Request",
            "text/plain",
            "Invalid request line",
        )
        .await?;
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];

    // CORS preflight
    if method == "OPTIONS" {
        send_cors_preflight(socket).await?;
        return Ok(());
    }

    if method == "GET" && (path == "/health" || path == "/") {
        let json = serde_json::json!({
            "status": "ok",
            "app": "Nodera",
            "version": "0.4.0",
            "vault": vault_name,
        });
        send_response(socket, 200, "OK", "application/json", &json.to_string()).await?;
        return Ok(());
    }

    if method == "POST" && path == "/clip" {
        // Find end of HTTP headers (\r\n\r\n or \n\n)
        let header_end = if let Some(pos) = find_subsequence(&buf[..n], b"\r\n\r\n") {
            pos + 4
        } else if let Some(pos) = find_subsequence(&buf[..n], b"\n\n") {
            pos + 2
        } else {
            send_response(
                socket,
                400,
                "Bad Request",
                "text/plain",
                "Malformed headers",
            )
            .await?;
            return Ok(());
        };

        // Parse Content-Length
        let content_length: usize = lines
            .filter_map(|l| {
                let lower = l.to_lowercase();
                if lower.starts_with("content-length:") {
                    l.split(':').nth(1).and_then(|v| v.trim().parse().ok())
                } else {
                    None
                }
            })
            .next()
            .unwrap_or(0);

        let mut body_bytes = buf[header_end..n].to_vec();
        // Read remaining body bytes if needed
        while body_bytes.len() < content_length {
            let mut chunk = vec![0u8; 4096];
            let read_n = socket.read(&mut chunk).await?;
            if read_n == 0 {
                break;
            }
            body_bytes.extend_from_slice(&chunk[..read_n]);
        }

        let payload: WebClipPayload = match serde_json::from_slice(&body_bytes) {
            Ok(p) => p,
            Err(e) => {
                let err_msg = format!("JSON parse error: {}", e);
                send_response(
                    socket,
                    400,
                    "Bad Request",
                    "application/json",
                    &serde_json::json!({ "error": err_msg }).to_string(),
                )
                .await?;
                return Ok(());
            }
        };

        // Create the clipped note on disk
        match save_clip_to_vault(vault_root, &payload) {
            Ok(rel_path) => {
                if let Some(tx) = notify_tx {
                    let _ = tx.send(rel_path.clone());
                }
                let resp = serde_json::json!({
                    "status": "ok",
                    "note_path": rel_path.to_string_lossy().replace('\\', "/"),
                });
                send_response(socket, 200, "OK", "application/json", &resp.to_string()).await?;
            }
            Err(e) => {
                let err_msg = format!("Failed to save note: {}", e);
                send_response(
                    socket,
                    500,
                    "Internal Server Error",
                    "application/json",
                    &serde_json::json!({ "error": err_msg }).to_string(),
                )
                .await?;
            }
        }
        return Ok(());
    }

    send_response(socket, 404, "Not Found", "text/plain", "Endpoint not found").await?;
    Ok(())
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

async fn send_cors_preflight(socket: &mut tokio::net::TcpStream) -> Result<(), std::io::Error> {
    let response = "HTTP/1.1 200 OK\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
Content-Length: 0\r\n\
Connection: close\r\n\r\n";
    socket.write_all(response.as_bytes()).await?;
    socket.flush().await?;
    Ok(())
}

async fn send_response(
    socket: &mut tokio::net::TcpStream,
    status_code: u16,
    status_text: &str,
    content_type: &str,
    body: &str,
) -> Result<(), std::io::Error> {
    let body_bytes = body.as_bytes();
    let header = format!(
        "HTTP/1.1 {} {}\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
Content-Type: {}\r\n\
Content-Length: {}\r\n\
Connection: close\r\n\r\n",
        status_code,
        status_text,
        content_type,
        body_bytes.len()
    );

    socket.write_all(header.as_bytes()).await?;
    socket.write_all(body_bytes).await?;
    socket.flush().await?;
    Ok(())
}

/// Creates a sanitized Markdown note in `<vault>/Clippings/` from a web clip payload.
pub fn save_clip_to_vault(
    vault_root: &Path,
    payload: &WebClipPayload,
) -> Result<PathBuf, std::io::Error> {
    let clippings_dir = vault_root.join("Clippings");
    if !clippings_dir.exists() {
        std::fs::create_dir_all(&clippings_dir)?;
    }

    let sanitized_title = nodera_core::sanitize_filename(&payload.title);
    let note_name = if sanitized_title.trim().is_empty() {
        "Web Clip".to_string()
    } else {
        sanitized_title
    };

    // Find non-conflicting filename
    let mut target_filename = format!("{}.md", note_name);
    let mut counter = 1;
    while clippings_dir.join(&target_filename).exists() {
        target_filename = format!("{} {}.md", note_name, counter);
        counter += 1;
    }

    let full_path = clippings_dir.join(&target_filename);
    let rel_path = PathBuf::from("Clippings").join(&target_filename);

    let now_iso = Utc::now().to_rfc3339();
    let mut tags = vec!["web-clip".to_string()];
    if let Some(ref custom_tags) = payload.tags {
        for t in custom_tags {
            let clean = t.trim().trim_start_matches('#');
            if !clean.is_empty()
                && !tags
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(clean))
            {
                tags.push(clean.to_string());
            }
        }
    }

    let mut md = String::new();
    md.push_str("---\n");
    md.push_str(&format!(
        "title: \"{}\"\n",
        payload.title.replace('"', "\\\"")
    ));
    md.push_str(&format!("source_url: \"{}\"\n", payload.url));
    if let Some(ref author) = payload.author {
        md.push_str(&format!("author: \"{}\"\n", author.replace('"', "\\\"")));
    }
    md.push_str(&format!("clipped_at: \"{}\"\n", now_iso));
    md.push_str("tags:\n");
    for tag in &tags {
        md.push_str(&format!("  - {}\n", tag));
    }
    md.push_str("---\n\n");

    md.push_str(&format!("# {}\n\n", payload.title));
    md.push_str(&format!(
        "> Clipped from [{}]({})\n\n",
        payload.url, payload.url
    ));

    if let Some(ref selected) = payload.selected_text {
        if !selected.trim().is_empty() {
            md.push_str("## Highlighted Excerpt\n\n");
            for line in selected.lines() {
                md.push_str(&format!("> {}\n", line));
            }
            md.push_str("\n---\n\n");
        }
    }

    md.push_str(&payload.content);
    md.push('\n');

    nodera_core::atomic_write_str(&full_path, &md)
        .map_err(|e| std::io::Error::other(e.to_string()))?;

    Ok(rel_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_web_clipper_http_lifecycle() {
        let dir = tempdir().unwrap();
        let vault_root = dir.path().to_path_buf();

        let (notify_tx, mut notify_rx) = mpsc::unbounded_channel();

        // Bind to ephemeral port (0)
        let handle = start_web_clipper(
            vault_root.clone(),
            "ResearchVault".to_string(),
            0,
            Some(notify_tx),
        )
        .await
        .unwrap();

        let port = handle.port;
        assert!(port > 0);

        // 1. Test GET /health
        let mut stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .unwrap();
        stream
            .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();

        let mut buf = vec![0u8; 2048];
        let n = stream.read(&mut buf).await.unwrap();
        let resp = String::from_utf8_lossy(&buf[..n]);
        assert!(resp.contains("HTTP/1.1 200 OK"));
        assert!(resp.contains("\"status\":\"ok\""));
        assert!(resp.contains("\"vault\":\"ResearchVault\""));

        // 2. Test POST /clip
        let clip_json = serde_json::json!({
            "url": "https://example.com/ai-paper",
            "title": "Quantum Deep Learning",
            "content": "Full article markdown body about qubits.",
            "selected_text": "Key discovery in neural quantum circuits.",
            "tags": ["quantum", "ai"],
            "author": "Dr. Smith"
        })
        .to_string();

        let req = format!(
            "POST /clip HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            clip_json.len(),
            clip_json
        );

        let mut stream2 = tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .unwrap();
        stream2.write_all(req.as_bytes()).await.unwrap();

        let mut buf2 = vec![0u8; 2048];
        let n2 = stream2.read(&mut buf2).await.unwrap();
        let resp2 = String::from_utf8_lossy(&buf2[..n2]);
        assert!(resp2.contains("HTTP/1.1 200 OK"));
        assert!(resp2.contains("\"status\":\"ok\""));
        assert!(resp2.contains("Clippings/Quantum Deep Learning.md"));

        // Check notify channel received path
        let received_path = notify_rx.try_recv().unwrap();
        assert_eq!(
            received_path.to_string_lossy().replace('\\', "/"),
            "Clippings/Quantum Deep Learning.md"
        );

        // Check note was created on disk with frontmatter
        let note_file = vault_root
            .join("Clippings")
            .join("Quantum Deep Learning.md");
        assert!(note_file.exists());
        let saved_content = std::fs::read_to_string(&note_file).unwrap();
        assert!(saved_content.contains("title: \"Quantum Deep Learning\""));
        assert!(saved_content.contains("source_url: \"https://example.com/ai-paper\""));
        assert!(saved_content.contains("- quantum"));
        assert!(saved_content.contains("Highlighted Excerpt"));
        assert!(saved_content.contains("Full article markdown body about qubits."));
    }
}
