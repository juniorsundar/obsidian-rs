use notify::EventKind;

mod config;
mod watcher;

fn main() {
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter("RUST_LOG")
            .write_style("LOG_STYLE"),
    );

    match watcher::initialize_watcher() {
        Ok((_watcher, rx)) => {
            log::info!("Watcher initialized successfully. Waiting for events...");
            for res in rx {
                match res {
                    Ok(event) => match event.kind {
                        EventKind::Create(_) => watcher::create_callback(&event),
                        EventKind::Remove(_) => watcher::remove_callback(&event),
                        EventKind::Modify(_) => watcher::modify_callback(&event),
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
