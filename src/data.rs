use crate::config::{self, AppConfig};
use crate::util;

use serde::Deserialize;
use std::{
    env,
    error::Error,
    fmt,
    fs::File,
    io::BufRead,
    io::BufReader,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

static DEFAULT_DATA_DIR: &str = "obsidian-rs";

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

pub fn get_data_path(config: &AppConfig) -> Result<PathBuf, Box<dyn Error>> {
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

    data_path.push(DEFAULT_DATA_DIR);
    data_path.push(vault_name);

    Ok(data_path)
}

#[derive(Deserialize, Debug, Default)]
pub struct NodeData {
    pub id: Option<PathBuf>,
    pub front_matter: Option<FrontMatter>,
}

#[derive(Deserialize, Debug, Default)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub github: Option<String>,
    pub created: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub authors: Option<Vec<String>>,
}

impl fmt::Display for FrontMatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output_parts = Vec::new();

        if let Some(title) = &self.title {
            output_parts.push(format!("Title: {}", title));
        }

        if let Some(github) = &self.github {
            output_parts.push(format!("github: {}", github));
        }

        if let Some(created_dates) = &self.created {
            if !created_dates.is_empty() {
                output_parts.push(format!("Created: {}", created_dates.join(", ")));
            }
        }

        if let Some(tags) = &self.tags {
            if !tags.is_empty() {
                output_parts.push(format!("Tags: {}", tags.join(", ")));
            }
        }

        if let Some(authors) = &self.authors {
            if !authors.is_empty() {
                output_parts.push(format!("Authors: {}", authors.join(", ")));
            }
        }
        write!(f, "{}", output_parts.join("\n"))
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn traverse_vault(vault_path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let walker = WalkDir::new(vault_path).into_iter();
    let mut files = Vec::<PathBuf>::new();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let current_entry = entry?;
        let path_to_current_entry = current_entry.path();

        if !path_to_current_entry.is_file() {
            continue;
        }
        files.push(path_to_current_entry.to_path_buf());
        log::debug!("{}", current_entry.path().display());
    }
    Ok(files)
}

pub fn parse_yaml_front_matter(file_path: &Path) -> Result<Option<FrontMatter>, Box<dyn Error>> {
    let file = File::open(file_path)
        .map_err(|e| format!("Error opening file '{}': {}", file_path.display(), e))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    match lines.next() {
        Some(Ok(line)) if line.trim() == "---" => {}
        Some(Err(e)) => {
            return Err(format!(
                "IO Error reading first line of '{}': {}",
                file_path.display(),
                e
            )
            .into());
        }
        _ => {
            return Ok(None);
        }
    }

    let mut yaml_content = String::new();
    let mut end_delimiter = false;
    for line_result in lines {
        let line = line_result
            .map_err(|e| format!("IO Error reading file '{}': {}", file_path.display(), e))?;
        if line.trim() == "---" {
            end_delimiter = true;
            break;
        }
        yaml_content.push_str(&line);
        yaml_content.push('\n');
    }

    if !end_delimiter {
        return Err(format!(
            "Malformed front matter in '{}': closing '---' delimiter not found.",
            file_path.display()
        )
        .into());
    }

    if yaml_content.trim().is_empty() {
        return Ok(None);
    }

    let mut data: FrontMatter = if yaml_content.trim().is_empty() {
        FrontMatter::default()
    } else {
        serde_yaml::from_str::<FrontMatter>(&yaml_content).map_err(|e| -> Box<dyn Error> {
            format!(
                "Failed to parse YAML front matter in '{}': {}",
                file_path.display(),
                e
            )
            .into()
        })?
    };

    if data.title.is_none() {
        if let Some(file_stem) = file_path.file_stem() {
            if let Some(stem_str) = file_stem.to_str() {
                log::debug!(
                    "Front matter title missing in '{}', using file stem: '{}'",
                    file_path.display(),
                    stem_str
                );
                data.title = Some(stem_str.to_string());
            } else {
                log::warn!(
                    "Front matter title missing in '{}', but file stem is not valid UTF-8.",
                    file_path.display()
                );
            }
        } else {
            log::warn!(
                "Front matter title missing in '{}', but could not extract file stem (e.g., path is root or invalid).",
                file_path.display()
            );
        }
    }
    Ok(Some(data))
}

/// Check to see if caching database exists
fn cache_exists() {}

/// Build cache with the files in the vault
fn build_cache() {}

/// Parse through entries in database to see if all are present
pub fn invalidate_cache(nodes: &Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    for node in nodes {
        if !exists_in_cache(node) {
            add_to_cache(node)?;
        } else {
            update_in_cache(node)?;
        }
    }
    Ok(())
}

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
fn update_in_cache(entry: &Path) -> Result<(), Box<dyn Error>> {
    Ok(())
}
