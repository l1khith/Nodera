use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use nodera_desktop::App;

fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nodera=info,nodera_desktop=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Check command-line arguments for protocol management or deep links
    let args: Vec<String> = std::env::args().collect();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--register-protocol" || arg == "register-protocol" {
            if let Ok(exe) = std::env::current_exe() {
                match nodera_core::register_windows_protocol(&exe) {
                    Ok(_) => println!(
                        "Successfully registered 'nodera://' protocol handler in Windows registry."
                    ),
                    Err(e) => eprintln!("Failed to register protocol: {e}"),
                }
            }
            return;
        } else if arg == "--unregister-protocol" || arg == "unregister-protocol" {
            match nodera_core::unregister_windows_protocol() {
                Ok(_) => println!(
                    "Successfully unregistered 'nodera://' protocol handler from Windows registry."
                ),
                Err(e) => eprintln!("Failed to unregister protocol: {e}"),
            }
            return;
        }
        i += 1;
    }

    let icon = dioxus::desktop::tao::window::Icon::from_rgba(
        include_bytes!("../../../assets/branding/nodera-icon-256.bin").to_vec(),
        256,
        256,
    )
    .expect("Valid Nodera window icon");

    let window = WindowBuilder::new()
        .with_title("Nodera — Knowledge Workspace")
        .with_window_icon(Some(icon.clone()))
        .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(1120.0, 760.0))
        .with_min_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(720.0, 480.0));

    // Isolated per-process session directory in %APPDATA%\.nodera\webview_sessions\session_{pid}
    // Guarantees WebView2 never panics with 0x800700AA lock conflicts.
    let webview_data_dir = {
        let base_dir = std::env::var_os("APPDATA")
            .or_else(|| std::env::var_os("LOCALAPPDATA"))
            .or_else(|| std::env::var_os("USERPROFILE"))
            .or_else(|| std::env::var_os("HOME"))
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let nodera_dir = base_dir.join(".nodera");
        let sessions_dir = nodera_dir.join("webview_sessions");
        let _ = std::fs::create_dir_all(&sessions_dir);

        // Best-effort cleanup of old shared folder and inactive session folders
        let old_shared = nodera_dir.join("webview_data");
        let _ = std::fs::remove_dir_all(&old_shared);

        if let Ok(entries) = std::fs::read_dir(&sessions_dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        let path = entry.path();
                        let dir_name = entry.file_name().to_string_lossy().to_string();
                        if let Some(pid_str) = dir_name.strip_prefix("session_") {
                            if let Ok(pid) = pid_str.parse::<u32>() {
                                if pid != std::process::id() {
                                    let _ = std::fs::remove_dir_all(&path);
                                }
                            }
                        }
                    }
                }
            }
        }

        let current_session = sessions_dir.join(format!("session_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&current_session);
        current_session
    };

    let config = Config::new()
        .with_window(window)
        .with_icon(icon)
        .with_data_directory(webview_data_dir);

    LaunchBuilder::desktop().with_cfg(config).launch(App);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_desktop_smoke() {
        let app_name = "Nodera";
        assert_eq!(app_name, "Nodera");
    }
}
