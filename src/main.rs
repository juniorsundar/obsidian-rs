use config::AppConfig;

mod config;
mod data;
mod util;
mod watcher;

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

    let vault_path = match config::get_root_workspace_path(&config) {
        Some(path) => path,
        None => {
            log::error!("Vault path not found in configuration.");
            std::process::exit(1);
        }
    };

    let _vault_content = match data::traverse_vault(&vault_path.as_path()) {
        Err(e) => {
            log::error!("Error in path_traversal: {}", e);
            std::process::exit(1);
        }
        _ => {}
    };

    if let Err(e) = watcher::run_watcher(&vault_path) {
        log::error!("Watcher failed to run: {}", e);
        std::process::exit(1);
    } else {
        log::info!("Watcher finished successfully.");
    }
}
