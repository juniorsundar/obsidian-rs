use crate::config::{self, AppConfig};
use crate::util;

use serde::Deserialize;
use std::thread::current;
use std::{env, error::Error, path::{PathBuf, Path}};
use walkdir::{DirEntry, WalkDir};

fn get_local_data_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        env::var("LOCALAPPDATA").ok().map(PathBuf::from)
    }
    #[cfg(unix)]
    {
        env::var("XDG_DATA_HOME")
            .ok()
            .filter(|s| !s.is_empty()) // Make sure it's not empty if set
            .map(PathBuf::from)
            .or_else(|| {
                if let Some(mut home) = util::get_home_dir() {
                    home.push(".local");
                    home.push("share");
                    Some(home)
                } else {
                    None
                }
            })
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

fn get_data_path() -> Result<PathBuf, Box<dyn Error>> {
    let config: AppConfig = config::extract_config()?;
    let mut data_path = get_local_data_dir()
        .ok_or_else(|| -> Box<dyn Error> { Box::from("Local data folder not found.") })?;
    let root_workspace_path = config::get_root_workspace_path(&config)
        .ok_or("Failed to get Root Workspace Path from Config.")?;

    let file_name_os_str = root_workspace_path
        .file_name()
        .ok_or_else(|| -> Box<dyn Error> {
            Box::from(format!(
                "Invalid configuration: Workspace path '{}' has no final component (filename).",
                root_workspace_path.display()
            ))
        })?;

    let vault_name = file_name_os_str.to_str().ok_or_else(|| -> Box<dyn Error> {
        Box::from(format!(
            "Invalid configuration: Vault name in path '{}' contains non-UTF-8 characters.",
            root_workspace_path.display()
        ))
    })?;

    data_path.push(vault_name);

    Ok(data_path)
}

#[derive(Deserialize, Debug)]
pub struct NodeData {
    pub file_path: Option<PathBuf>,
    pub front_matter: Option<FrontMatter>,
}

#[derive(Deserialize, Debug)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub created: Option<String>,
    pub tags: Option<Vec<String>>,
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn traverse_vault(vault_path: &Path) -> Result<(), Box<dyn Error>> {
    let walker = WalkDir::new(vault_path).into_iter();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let current_entry = entry?;
        let path_to_current_entry = current_entry.path();

        if !path_to_current_entry.is_file() {
            continue;
        } 

        log::info!("{}", current_entry.path().display());

        if !exists_in_cache(path_to_current_entry) {
            add_to_cache(path_to_current_entry)?;
        } else {
            update_in_cache(path_to_current_entry)?;
        }
    }
    Ok(())
}

/// Extract yaml front matter of provided file and store as struct
fn parse_yaml_front_matter() {}

/// Check to see if caching database exists
fn cache_exists() {}

/// Build cache with the files in the vault
fn build_cache() {}

/// Parse through entries in database to see if all are present
fn invalidate_cache() {}

/// Exists in cache?
fn exists_in_cache(entry: &Path) -> bool {
    true
}

/// Add entry to cache
fn add_to_cache(entry: &Path) -> Result<(), Box<dyn Error>> {
    Ok(())
}

/// Remove entry from cache
fn remove_from_cache() {}

/// Update entry in cache
fn update_in_cache(entry: &Path) -> Result<(), Box<dyn Error>>{
    Ok(())
}
