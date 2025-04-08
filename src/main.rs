use log;
use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind,
};
use std::{
    borrow::Cow,
    error::Error,
    path::Path,
    sync::mpsc::Receiver,
};

mod config;
use crate::config::AppConfig;

fn watch_setup(
    path: &Path,
) -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    if !path.exists() {
        log::error!("Path does not exist or is not accessible: {:?}", path);
        return Err(notify::Error::path_not_found());
    }
    watcher.watch(path, RecursiveMode::Recursive)?;
    Ok((watcher, rx))
}

fn initialize_watcher() -> Result<(RecommendedWatcher, Receiver<notify::Result<Event>>), Box<dyn Error>> {
    // 1. Get config path (Uses config::get_config)
    let config_path = config::get_config().ok_or("Failed to expand config_path!")?;

    // 2. Extract config (Uses config::extract_config)
    let config: AppConfig = config::extract_config(&config_path)?;
    log::info!("Config loaded successfully: {:?}", config);

    // 3. Expand tilde path (Uses config::expand_tilde)
    let root_path = Path::new(&config.workspace.root);
    let expanded_cow: Cow<'_, Path> = config::expand_tilde(root_path).ok_or_else(|| {
        format!(
            "Failed to expand home directory for path '{}'. Check HOME/USERPROFILE env var.",
            config.workspace.root // Access workspace.root from the loaded config
        )
    })?;
    let expanded_root: &Path = expanded_cow.as_ref();
    log::info!("Expanded root path to watch: {:?}", expanded_root);

    // 4. Set up the file watcher by calling the modified function
    let (watcher, rx) = watch_setup(expanded_root).map_err(|e| Box::new(e) as Box<dyn Error>)?;
    log::info!("Successfully watching path: {:?}", expanded_root);

    Ok((watcher, rx))
}

fn main() {
    // simple_logger::init_with_level(log::Level::Info).expect("Failed to initialize logger");
    simple_logger::init_with_env().unwrap();

    match initialize_watcher() {
        Ok((_watcher, rx)) => {
            log::info!("Watcher initialized successfully. Waiting for events...");
            for res in rx {
                match res {
                    Ok(event) => match event.kind {
                        EventKind::Create(_) => create_callback(&event),
                        EventKind::Remove(_) => remove_callback(&event),
                        EventKind::Modify(_) => modify_callback(&event),
                        _ => {}
                    },
                    Err(error) => log::error!("Error receiving file event: {error:?}"),
                }
            }
            log::info!("Event loop finished.");
        }
        Err(e) => {
            log::error!("Failed to initialize watcher: {}", e);
            std::process::exit(1);
        }
    }
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
