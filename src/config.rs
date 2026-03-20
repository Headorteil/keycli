//!All functions needed to manipulate config files

use crate::consts::{DEFAULT_CONF_FILE, PATH_HEADER, SHA_HEADER};
use anyhow::{Context, Result, anyhow};
use regex::Regex;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::LazyLock;
use std::{env, fs};

static CONFIG_HEADERS_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        "^{PATH_HEADER}(?P<path>.*)\r?\n{SHA_HEADER}(?P<sha>.*)\r?\n"
    ))
    .unwrap()
});

/// Check if the template associated to a config is valid
pub fn check_template(config: &str) -> Result<()> {
    let captures = CONFIG_HEADERS_REGEX.captures(config);
    let (path, sha) = match &captures {
        Some(captures) => (&captures["path"], &captures["sha"]),
        None => return Ok(()),
    };
    log::debug!("Found template reference in the config: {path} {sha}");

    let content = fs::read(path).with_context(|| format!("Failed to read template file {path}, you can run with '--no-check-template' or remove the keycli comments in the config file"))?;
    let hash = hex::encode(Sha256::digest(&content));
    if hash != sha {
        return Err(anyhow!(
            "Associated template file was changed, you can update your config by running 'keycli init' while being in the dir it lays, run with '--no-check-template' or remove the keycli comments in the config file"
        ));
    }
    Ok(())
}

/// Search for the first config file in the parent directories
pub fn search_config() -> Option<PathBuf> {
    let dir = env::current_dir().ok()?;
    for ancestor in dir.ancestors() {
        let candidate = ancestor.join(DEFAULT_CONF_FILE);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}
