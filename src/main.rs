mod config;
mod data;
mod util;
mod watcher;

use config::AppConfig;
use data::NodeData;

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

    let data = match data::get_data_path(&config) {
        Err(e) => {
            log::error!("Problem retrieving data-path: {}", e);
            std::process::exit(1);
        }
        Ok(path) => {
            log::info!("{}", path.display());
            path
        }
    };

    let cache = match data::get_cache(&data) {
        Err(e) => {
            log::error!("Problem retrieving cache db: {}", e);
            std::process::exit(1);
        }
        Ok(cache_conn) => {
            cache_conn
        }
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

    let _cache_state = match data::invalidate_cache(&vault_content, &vault_path, &cache) {
        Err(e) => {
            log::error!("Error in invalidation: {}", e);
            std::process::exit(1);
        }
        _ => {}
    };

    let mut nodes: Vec<NodeData> = Vec::new();
    for file in vault_content {
        match data::parse_yaml_front_matter(&file.as_path()) {
            Err(_) => {}
            Ok(fm_opt) => match fm_opt {
                Some(fm) => {
                    let rel_path = util::get_relative_path(&file, &vault_path).unwrap();
                    let node = NodeData {
                        id: Some(rel_path),
                        front_matter: Some(fm),
                    };
                    log::info!("{}", node);
                    nodes.push(node);
                }
                None => {}
            },
        };
    }

    // ------

    if let Err(e) = watcher::run_watcher(&vault_path) {
        log::error!("Watcher failed to run: {}", e);
        std::process::exit(1);
    } else {
        log::info!("Watcher finished successfully.");
    }
}
