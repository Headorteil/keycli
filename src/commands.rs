//! All the commands to manipulate the environment and the keyring

use crate::consts::{LINE_ENDING, PATH_HEADER, SHA_HEADER};
use crate::secrets;
use crate::secrets::Secret;
use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Password};
use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

/// Save a secret list to the OS keyring from
pub fn save(
    app_name: Option<String>,
    secrets: Vec<Secret>,
    interactive: bool,
    force: bool,
) -> Result<Vec<Secret>> {
    let mut new_secrets = Vec::<Secret>::new();
    for secret in secrets {
        if interactive {
            let resolved_secret = secret.to_keycli_str()?;
            let new_secret_str = confirm_or_edit("The secret full path is", &resolved_secret)?;
            let new_secret = Secret::new(app_name.clone(), &new_secret_str)?;
            if !new_secret.exists()?
                || Confirm::new()
                    .with_prompt(format!(
                        "{} already exist in the keyring, replace it?",
                        new_secret.to_keyring_str()
                    ))
                    .default(false)
                    .interact()?
            {
                let value = Password::new()
                    .with_prompt(format!(
                        "Input the value of '{}'",
                        new_secret.to_keycli_str()?
                    ))
                    .interact()?;
                new_secret.push(&value)?;
            }
            new_secrets.push(new_secret);
        } else {
            let new_secret = secret;
            if !force && new_secret.exists()? {
                log::warn!(
                    "{} already exist in the keyring and force is false, not pushing it",
                    new_secret.to_keyring_str()
                );
            } else {
                let value = env::var(&new_secret.env).context(format!(
                    "Environment variable {} is not defined",
                    new_secret.env
                ))?;
                new_secret.push(&value)?;
            }
            new_secrets.push(new_secret);
        }
    }
    Ok(new_secrets)
}

/// Clear the OS keyring from a secret list
pub fn clear(secrets: Vec<Secret>, interactive: bool) -> Result<()> {
    for secret in secrets {
        if !secret.exists()? {
            log::warn!("{} does not exist in the keyring", secret.to_keyring_str());
            continue;
        }
        if !interactive
            || Confirm::new()
                .with_prompt(format!(
                    "Are you sure to delete {}?",
                    secret.to_keyring_str()
                ))
                .default(false)
                .interact()?
        {
            secret.clear()?;
            continue;
        }
    }
    Ok(())
}

/// Build a sourcable string from a secret list which would load the desired
/// secrets in the environment
pub fn load(secrets: Vec<Secret>) -> Result<String> {
    let mut result = String::new();
    for (env, password) in secrets::build_env(secrets)? {
        write!(&mut result, "export {env}={password}{LINE_ENDING}")?;
        log::debug!("Processed {env}");
    }
    Ok(result)
}

/// Build a sourcable string from a secret list which would unload the desired
/// environment variables
pub fn unload(secrets: Vec<Secret>) -> Result<String> {
    let mut result = String::new();
    for secret in secrets {
        write!(&mut result, "unset {}{LINE_ENDING}", secret.env)?;
        log::debug!("Processed {}", secret.env);
    }
    Ok(result)
}

/// Exec a binary while loading the environment variables specified in a secret list
pub fn exec(secrets: Vec<Secret>, binary: &str, args: Vec<String>) -> Result<()> {
    let env_map = secrets::build_env(secrets)?;
    Command::new(binary).envs(env_map).args(&args).status()?;
    Ok(())
}

/// Generate a config file from a secret list
pub fn init(secrets: Vec<Secret>, template: Option<PathBuf>) -> Result<String> {
    let mut result = String::new();
    if let Some(path) = template {
        let canon_path = std::fs::canonicalize(&path)?;
        let content = fs::read(path.clone())?;
        let hash = hex::encode(Sha256::digest(&content));
        write!(
            &mut result,
            "{PATH_HEADER}{}{LINE_ENDING}",
            canon_path.display()
        )?;
        write!(&mut result, "{SHA_HEADER}{hash}{LINE_ENDING}")?;
    };
    write!(&mut result, "{}", secrets::init_str(secrets)?)?;
    Ok(result)
}

/// Prompt for a confirmation and allow an edit in the case of a non confirmation
fn confirm_or_edit(prompt: &str, value: &str) -> Result<String> {
    let confirmed = Confirm::new()
        .with_prompt(format!("{prompt}: '{value}'?"))
        .default(true)
        .interact()?;

    if confirmed {
        Ok(value.to_string())
    } else {
        Ok(Input::new()
            .with_prompt("Enter new value")
            .interact_text()?)
    }
}
