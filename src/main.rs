use env_logger;
use log;
use notify::{event::ModifyKind, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use std::{fs, path::Path};
use toml;

mod path_expander;

#[derive(Deserialize, Debug)]
struct AppConfig {
    workspace: Workspace,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    // name: String,
    root: String,
    // port: u16,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    if let Ok(config) = extract_config(&args) {
        log::info!("Config loaded successfully: {:?}", config);
        let root_path = Path::new(&config.workspace.root);
        match path_expander::expand_tilde(root_path) {
            Some(expanded_cow) => {
                let expanded_root: &Path = expanded_cow.as_ref();
                log::info!("Expanded root path to watch: {:?}", expanded_root);
                if let Err(error) = watch(expanded_root) {
                    log::error!("Error setting up file watcher: {error:?}");
                }
            }
            None => {
                log::error!(
                    "Failed to expand home directory for path '{}'. Check HOME/USERPROFILE env var.",
                    config.workspace.root
                );
                return;
            }
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
            Ok(event) => {
                match event.kind {
                    EventKind::Create(_) => {
                        create_callback(&event);
                    }
                    EventKind::Remove(_) => {
                        remove_callback(&event);
                    }
                    EventKind::Modify(_) => {
                        modify_callback(&event);
                    }
                    // EventKind::Access(_) => {access_callback(&event);},
                    // _ => {other_event_callback(&event);}
                    _ => {}
                };
            }
            Err(error) => log::error!("Error: {error:?}"),
        }
    }

    Ok(())
}

fn create_callback(event: &Event) {
    log::info!("--- Create Event ---");
    log::info!("  Paths involved: {}", event.paths.len());
    for path in &event.paths {
        // Usually just one path for Create
        log::info!("   -> Created: {}", path.display());
    }
}

fn modify_callback(event: &Event) {
    log::info!("--- Modify Event ---");
    log::info!("  Paths involved: {}", event.paths.len());

    // Check specifically for rename events if you want different logging
    if matches!(event.kind, EventKind::Modify(ModifyKind::Name(_))) {
        if event.paths.len() == 2 {
            // Likely RenameMode::Both (source and destination)
            // Note: notify doesn't guarantee the order of paths[0] and paths[1]
            log::info!("   -> Renamed/Moved From: {}", event.paths[0].display());
            log::info!("   -> Renamed/Moved To:   {}", event.paths[1].display());
        } else {
            // Likely RenameMode::From or RenameMode::To (separate events)
            for path in &event.paths {
                log::info!("   -> Modified Part: {}", path.display());
            }
        }
    } else {
        // Other modifications (data, metadata)
        for path in &event.paths {
            // Usually just one path for Data/Metadata changes
            log::info!("   -> Edited: {}", path.display());
        }
    }
}

fn remove_callback(event: &Event) {
    log::info!("--- Remove Event ---");
    log::info!("  Paths involved: {}", event.paths.len());
    for path in &event.paths {
        // Usually just one path for Remove
        log::info!("   -> Removed: {}", path.display());
    }
}

// fn access_callback(event: &Event) {
//     log::info!("--- Access Event ---");
//     log::info!("  Paths involved: {}", event.paths.len());
//     for path in &event.paths {
//          // Usually just one path for Access
//          log::info!("   -> Accessed: {}", path.display());
//     }
// }

// fn other_event_callback(event: &Event) {
//     // Catch-all for Any or Other kinds
//     log::info!("--- Other/Unknown Event ---");
//     log::info!("  Kind: {:?}", event.kind);
//     log::info!("  Paths involved: {}", event.paths.len());
//      for path in &event.paths {
//         log::info!("   -> Path: {}", path.display());
//     }
//      log::info!("  Attributes: {:?}", event.attrs);
//
// }
