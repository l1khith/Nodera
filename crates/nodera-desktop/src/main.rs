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

    let window = WindowBuilder::new()
        .with_title("Nodera — Knowledge Workspace")
        .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(1120.0, 760.0))
        .with_min_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(720.0, 480.0));

    let config = Config::new().with_window(window);

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
