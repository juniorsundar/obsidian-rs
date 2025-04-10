use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
    event::ModifyKind,
};
use std::{error::Error, path::Path, sync::mpsc::Receiver};

use crate::config::{self, AppConfig};

type WatchEventReceiver = Receiver<NotifyResult<Event>>;
type WatcherSetup = (RecommendedWatcher, WatchEventReceiver);

pub fn run_watcher() -> Result<(), Box<dyn Error>> {
    let config: AppConfig = config::extract_config()?;
    let root_workspace_path = config::get_root_workspace_path(&config)
        .ok_or("Failed to get Root Workspace Path from Config.")?;

    let (_watcher, rx) = watcher_setup(&root_workspace_path.as_path()).map_err(|e| Box::new(e) as Box<dyn Error>)?;
    log::info!("Successfully watching path: {:?}", root_workspace_path);

    for res in rx {
        match res {
            Ok(event) => callback_matcher(&event.kind, &event),
            Err(error) => log::error!("Error receiving file event: {error:?}"),
        }
    }
    Ok(())
}

fn watcher_setup(path: &Path) -> notify::Result<WatcherSetup> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    if !path.exists() {
        log::error!("Path does not exist or is not accessible: {:?}", path);
        return Err(notify::Error::path_not_found());
    }
    watcher.watch(path, RecursiveMode::Recursive)?;
    Ok((watcher, rx))
}

fn callback_matcher(event_kind: &EventKind, event: &Event) {
    match event_kind {
        EventKind::Create(_) => create_callback(event),
        EventKind::Remove(_) => remove_callback(event),
        EventKind::Modify(_) => modify_callback(event),
        EventKind::Access(_) => access_callback(event),
        _ => other_event_callback(event),
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

fn access_callback(event: &Event) {
    // log::info!("--- Access Event ---");
    // log::info!("  Paths involved: {}", event.paths.len());
    // for path in &event.paths {
    //      // Usually just one path for Access
    //      log::info!("   -> Accessed: {}", path.display());
    // }
}

fn other_event_callback(event: &Event) {
    //     // Catch-all for Any or Other kinds
    //     log::info!("--- Other/Unknown Event ---");
    //     log::info!("  Kind: {:?}", event.kind);
    //     log::info!("  Paths involved: {}", event.paths.len());
    //      for path in &event.paths {
    //         log::info!("   -> Path: {}", path.display());
    //     }
    //      log::info!("  Attributes: {:?}", event.attrs);
    //
}
