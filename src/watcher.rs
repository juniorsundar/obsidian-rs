use notify::{
    Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher, event::ModifyKind,
};
use std::{error::Error, path::PathBuf};

pub fn run_watcher(vault_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(vault_path, RecursiveMode::Recursive)?;
    log::info!("Successfully watching path: {:?}", vault_path);

    for res in rx {
        match res {
            Ok(event) => callback_matcher(&event.kind, &event),
            Err(error) => log::error!("Error receiving file event: {error:?}"),
        }
    }
    Ok(())
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
            // Note: notify doesn't guarantee the order of paths[0] and paths[1]
            log::info!("   -> Renamed/Moved From: {}", event.paths[0].display());
            log::info!("   -> Renamed/Moved To:   {}", event.paths[1].display());
        } else {
            for path in &event.paths {
                log::info!("   -> Modified Part: {}", path.display());
            }
        }
    } else {
        // Other modifications (data, metadata)
        for path in &event.paths {
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

fn access_callback(_event: &Event) {
    // log::info!("--- Access Event ---");
    // log::info!("  Paths involved: {}", event.paths.len());
    // for path in &event.paths {
    //      // Usually just one path for Access
    //      log::info!("   -> Accessed: {}", path.display());
    // }
}

fn other_event_callback(_event: &Event) {
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
