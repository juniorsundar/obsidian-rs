use env_logger;
use log;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use toml;

#[derive(Deserialize, Debug)]
struct AppConfig {
    workspace: Workspace,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    name: String,
    root: String,
    port: u16,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    if let Ok(config) = extract_config(&args) {
        log::info!("Config loaded successfully: {:?}", config);
        let expanded_root = shellexpand::tilde(&config.workspace.root).into_owned();
        log::info!("Expanded root path to watch: {}", expanded_root);
        if let Err(error) = watch(Path::new(&expanded_root)) {
            log::error!("Error setting up file watcher: {error:?}");
        }
    } else {
        log::error!("Failed to load or parse config file: {}", args);
    }
}

fn extract_config(args: &String) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_path = String::from(args);
    let config_content = fs::read_to_string(Path::new(&config_path))?;

    log::debug!("Read config content: {}", config_content);

    let config: AppConfig = toml::from_str(&config_content)?;
    Ok(config)
}

fn watch(path: &Path) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    //
    if !path.exists() {
        log::error!("Path does not exist or is not accessible: {:?}", path);
        return Err(notify::Error::path_not_found());
    }

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path, RecursiveMode::Recursive)?;
    log::info!("Successfully watching path: {:?}", path);

    for res in rx {
        match res {
            Ok(event) => log::info!("Change: {event:?}"),
            Err(error) => log::error!("Error: {error:?}"),
        }
    }

    Ok(())
}
