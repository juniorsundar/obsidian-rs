use config::AppConfig;

mod config;
mod data;
mod util;
mod watcher;

use rusqlite::{params, Connection, Result};

fn main() {
    env_logger::init_from_env(
        env_logger::Env::new()
            .filter("RUST_LOG")
            .write_style("LOG_STYLE"),
    );

    let config: AppConfig = match config::extract_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            log::error!("Failed to load configuration: {}", e);
            std::process::exit(1);
        }
    };

    let mut data = match data::get_data_path(&config) {
        Err(e) => {
            log::error!("Problem retrieving data-path: {}", e);
            std::process::exit(1);
        },
        Ok(path) => { log::info!("{}", path.display()); path }
    };

    data.push("cache.db3");
    let _conn = match Connection::open(data) {
        Err(e) => {
            log::error!("Problem creating database: {}", e);
            std::process::exit(1);
        },
        Ok(db) => db
    };


    // ------

    let vault_path = match config::get_root_workspace_path(&config) {
        Some(path) => path,
        None => {
            log::error!("Vault path not found in configuration.");
            std::process::exit(1);
        }
    };

    let vault_content = match data::traverse_vault(&vault_path.as_path()) {
        Err(e) => {
            log::error!("Error in path_traversal: {}", e);
            std::process::exit(1);
        }
        Ok(nodes) => nodes,
    };

    let _cache_state = match data::invalidate_cache(&vault_content) {
        Err(e) => {
            log::error!("Error in invalidation: {}", e);
            std::process::exit(1);
        }
        _ => {}
    };

    // ------

    if let Err(e) = watcher::run_watcher(&vault_path) {
        log::error!("Watcher failed to run: {}", e);
        std::process::exit(1);
    } else {
        log::info!("Watcher finished successfully.");
    }
}
