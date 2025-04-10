use serde::Deserialize;
use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

use crate::util;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub workspace: Workspace,
}

#[derive(Deserialize, Debug)]
pub struct Workspace {
    // name: String,
    pub root: String,
    // port: u16,
}

static DEFAULT_CONFIG_PATH: &str = "~/.config/obsidian-rs/config.toml";

fn get_config_path() -> Option<String> {
    if let Some(config_path_cow) = util::expand_tilde(Path::new(DEFAULT_CONFIG_PATH)) {
        config_path_cow.to_str().map(String::from)
    } else {
        None
    }
}

pub fn get_root_workspace_path(config: &AppConfig) -> Option<PathBuf> {
    let root_path = Path::new(&config.workspace.root);
    util::expand_tilde(root_path).map(|expanded_cow| expanded_cow.into_owned()) // Convert Cow -> PathBuf
}

pub fn extract_config() -> Result<AppConfig, Box<dyn Error>> {
    let config_path_str = get_config_path().ok_or("Failed to expand config path!")?;
    let config_path = Path::new(&config_path_str);

    let config_content = fs::read_to_string(config_path).map_err(|io_error| -> Box<dyn Error> {
        if io_error.kind() == io::ErrorKind::NotFound {
            let config_not_found_error = format!(
                "Configuration file not found at path: {}",
                config_path.display()
            );
            Box::from(config_not_found_error)
        } else {
            Box::new(io_error)
        }
    })?;

    log::debug!("Read config content: {}", config_content);

    let config: AppConfig = toml::from_str(&config_content)?;
    Ok(config)
}
