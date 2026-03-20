use crate::config::{check_template, search_config};
use crate::consts::{LINE_ENDING, TOOL_NAME};
use anyhow::{Context, Result, anyhow};
use heck::{ToLowerCamelCase, ToShoutySnakeCase};
use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;
use std::{env, fs};

use keyring::Entry;

/// Main struct, handles all the logic of the keycli secrets
pub struct Secret {
    pub service: String,
    pub username: String,
    pub env: String,
}

impl Secret {
    /// Compute a keycli secret string and extract from it the Secret
    pub fn new(opt_app_name: Option<String>, secret: &str) -> Result<Self> {
        let parts: Vec<&str> = secret.split(':').collect();
        let (service, username, env) = match parts.as_slice() {
            ["", secret] => {
                let (service, username, app_name) = split_secret(opt_app_name.as_deref(), secret)?;
                let env = format!("{app_name}_{username}").to_shouty_snake_case();
                (service, username, env)
            }
            [env, ""] | [env] => {
                let app_name = opt_app_name
                    .as_deref()
                    .ok_or_else(|| anyhow!("App name must be defined"))?;
                let app_name_shouty = format!("{app_name}_").to_shouty_snake_case();
                match env.strip_prefix(app_name_shouty.as_str()) {
                    Some(username) => (
                        format!("{TOOL_NAME}/{app_name}"),
                        username.to_lower_camel_case(),
                        env.to_string(),
                    ),
                    None => (
                        format!("{TOOL_NAME}/{app_name}"),
                        env.to_lower_camel_case(),
                        env.to_string(),
                    ),
                }
            }
            [env, secret] => {
                let (service, username, _) = split_secret(opt_app_name.as_deref(), secret)?;
                (service, username, env.to_string())
            }
            _ => return Err(anyhow!(r#"Too many ":" in secret "{secret}""#)),
        };
        Ok(Secret {
            service,
            username,
            env,
        })
    }

    /// Output a keycli secret string
    pub fn to_keycli_str(&self) -> Result<String> {
        let printable_service = self
            .service
            .strip_prefix(&format!("{TOOL_NAME}/"))
            .with_context(|| format!("Expected prefix '{TOOL_NAME}/' in '{}'", self.service))?;
        Ok(format!(
            "{}:{printable_service}/{}",
            self.env, self.username
        ))
    }

    /// Output the OS keyring path
    pub fn to_keyring_str(&self) -> String {
        format!("{}/{}", self.service, self.username)
    }

    /// Check if a secret exists in the OS keyring
    pub fn exists(&self) -> Result<bool> {
        let entry = Entry::new(&self.service, &self.username)?;
        match entry.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// Get secret from the OS keyring
    pub fn get(&self) -> Result<String> {
        log::debug!("Searching for {}", self.to_keyring_str());
        let entry = Entry::new(&self.service, &self.username)?;
        entry
            .get_password()
            .with_context(|| format!("Can't retrieve secret {}", self.to_keyring_str()))
    }

    /// Push secret to the OS keyring
    pub fn push(&self, password: &str) -> Result<()> {
        let entry = Entry::new(&self.service, &self.username)?;
        entry.set_password(password)?;
        log::info!("Secret {} was saved to the keyring", self.to_keyring_str());
        Ok(())
    }

    /// Clear secret from the OS keyring
    pub fn clear(&self) -> Result<()> {
        let entry = Entry::new(&self.service, &self.username)?;
        entry
            .delete_credential()
            .with_context(|| format!("Failed to delete secret {}", self.to_keyring_str()))?;
        log::info!("{} was deleted", self.to_keyring_str());
        Ok(())
    }
}

/// Compute a secret string (without the env part) and extract from it the service, username and app_name
pub fn split_secret(opt_app_name: Option<&str>, secret: &str) -> Result<(String, String, String)> {
    let parts: Vec<&str> = secret.split('/').collect();
    match parts.as_slice() {
        [app_name, username] => Ok((
            format!("{TOOL_NAME}/{app_name}"),
            username.to_string(),
            app_name.to_string(),
        )),
        [username] => {
            let app_name = opt_app_name.ok_or_else(|| anyhow!("App name must be defined"))?;
            if app_name.contains(':') || app_name.contains('/') {
                return Err(anyhow::anyhow!("app-name cannot contain ':' or '/'"));
            };

            Ok((
                format!("{TOOL_NAME}/{app_name}"),
                username.to_string(),
                app_name.to_string(),
            ))
        }
        _ => Err(anyhow!(r#"Too many "/" in secret "{secret}""#,)),
    }
}

/// Output a secret list from a config, secrets and extra secrets
pub fn parse_secrets(
    opt_app_name: Option<String>,
    arg_extra_secrets: Vec<String>,
    opt_config_file: Option<PathBuf>,
    arg_secrets: Vec<String>,
    overwrite: bool,
    check_tpl: bool,
    do_search_config: bool,
) -> Result<Vec<Secret>> {
    let mut raw_secrets = Vec::<String>::new();
    if arg_secrets.is_empty() {
        let content = match opt_config_file {
            Some(ref config_file) => fs::read_to_string(config_file)
                .with_context(|| format!("Failed to read file: '{}'", config_file.display()))?,
            None => {
                if do_search_config {
                    match search_config() {
                        Some(ref config_file) => {
                            fs::read_to_string(config_file).with_context(|| {
                                format!("Failed to read file: '{}'", config_file.display())
                            })?
                        }
                        None => String::new(),
                    }
                } else {
                    String::new()
                }
            }
        };
        if check_tpl {
            check_template(&content)?
        }
        let config_secrets: Vec<String> = content
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with("#"))
            .map(String::from)
            .collect();
        raw_secrets.extend(config_secrets);
        raw_secrets.extend(arg_extra_secrets);
    } else {
        raw_secrets.extend(arg_secrets);
    }
    if raw_secrets.is_empty() {
        return Err(anyhow!("No secrets provided"));
    }
    let mut parsed_secrets = Vec::<Secret>::new();
    for secret in raw_secrets {
        let secret = Secret::new(opt_app_name.clone(), &secret)?;
        if overwrite || env::var(&secret.env).is_err() {
            parsed_secrets.push(secret);
        } else {
            log::debug!(
                "Not loading {} as overwrite is false and the env is already defined",
                secret.env
            );
        }
    }
    Ok(parsed_secrets)
}

/// Build a config string from a Secret vector
pub fn init_str(secrets: Vec<Secret>) -> Result<String> {
    let mut result = String::new();
    for secret in &secrets {
        write!(&mut result, "{}{LINE_ENDING}", secret.to_keycli_str()?)?;
    }
    Ok(result)
}

/// Build an env hashmap from a Secret vector
pub fn build_env(secrets: Vec<Secret>) -> Result<HashMap<String, String>> {
    let mut env_map: HashMap<String, String> = HashMap::new();
    for secret in secrets {
        env_map.insert(secret.env.clone(), secret.get()?);
    }
    Ok(env_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_secret() {
        assert!(split_secret(None, "zaza").is_err());
        assert!(split_secret(Some("zouzou:zuzu"), "zaza").is_err());
        assert!(split_secret(Some("zouzou/zuzu"), "zaza").is_err());
        assert!(split_secret(Some("zonzon/zouzou/zuzu"), "zaza").is_err());

        assert_eq!(
            split_secret(None, "zozo/zaza").unwrap(),
            (
                String::from("keycli/zozo"),
                String::from("zaza"),
                String::from("zozo")
            )
        );
        assert_eq!(
            split_secret(Some("zuzu"), "zozo/zaza").unwrap(),
            (
                String::from("keycli/zozo"),
                String::from("zaza"),
                String::from("zozo")
            )
        );
        assert_eq!(
            split_secret(Some("zuzu"), "zaza").unwrap(),
            (
                String::from("keycli/zuzu"),
                String::from("zaza"),
                String::from("zuzu")
            )
        );
    }

    #[test]
    fn test_secret_new() {
        assert!(Secret::new(None, "ZOZO").is_err());
        assert!(Secret::new(None, ":zozo").is_err());
        assert!(Secret::new(None, "ZAZA:zozo").is_err());

        let secret = Secret::new(None, ":zozo/zaza").unwrap();
        assert_eq!(secret.service, String::from("keycli/zozo"));
        assert_eq!(secret.username, String::from("zaza"));
        assert_eq!(secret.env, String::from("ZOZO_ZAZA"));

        let secret = Secret::new(None, "ZUZU:zozo/zaza").unwrap();
        assert_eq!(secret.service, String::from("keycli/zozo"));
        assert_eq!(secret.username, String::from("zaza"));
        assert_eq!(secret.env, String::from("ZUZU"));

        let secret = Secret::new(Some(String::from("zonzon")), "ZUZU").unwrap();
        assert_eq!(secret.service, String::from("keycli/zonzon"));
        assert_eq!(secret.username, String::from("zuzu"));
        assert_eq!(secret.env, String::from("ZUZU"));

        let secret = Secret::new(Some(String::from("zonzon")), ":zozoZaza").unwrap();
        assert_eq!(secret.service, String::from("keycli/zonzon"));
        assert_eq!(secret.username, String::from("zozoZaza"));
        assert_eq!(secret.env, String::from("ZONZON_ZOZO_ZAZA"));

        let secret = Secret::new(Some(String::from("zonzon")), ":zozo_zaza").unwrap();
        assert_eq!(secret.service, String::from("keycli/zonzon"));
        assert_eq!(secret.username, String::from("zozo_zaza"));
        assert_eq!(secret.env, String::from("ZONZON_ZOZO_ZAZA"));

        let secret = Secret::new(Some(String::from("zonzon")), ":Zozo_Zaza").unwrap();
        assert_eq!(secret.service, String::from("keycli/zonzon"));
        assert_eq!(secret.username, String::from("Zozo_Zaza"));
        assert_eq!(secret.env, String::from("ZONZON_ZOZO_ZAZA"));

        let secret = Secret::new(Some(String::from("zonzon")), ":zozo/zouzou").unwrap();
        assert_eq!(secret.service, String::from("keycli/zozo"));
        assert_eq!(secret.username, String::from("zouzou"));
        assert_eq!(secret.env, String::from("ZOZO_ZOUZOU"));

        let secret = Secret::new(Some(String::from("zonzon")), "ZUZU:zouzou").unwrap();
        assert_eq!(secret.service, String::from("keycli/zonzon"));
        assert_eq!(secret.username, String::from("zouzou"));
        assert_eq!(secret.env, String::from("ZUZU"));

        let secret = Secret::new(Some(String::from("zonzon")), "ZUZU:zozo/zouzou").unwrap();
        assert_eq!(secret.service, String::from("keycli/zozo"));
        assert_eq!(secret.username, String::from("zouzou"));
        assert_eq!(secret.env, String::from("ZUZU"));
    }

    #[test]
    fn test_secret_resolve() {
        let secret = Secret::new(None, "zonzon:zozo/zaza").unwrap();
        assert_eq!(secret.to_keycli_str().unwrap(), "zonzon:zozo/zaza");
    }

    #[test]
    fn test_init_str() {
        let secret1 = Secret::new(None, "ZOUZOU:zozo/zaza").unwrap();
        let secret2 = Secret::new(None, "ZONZON:zuzu/zaza").unwrap();
        let secrets = vec![secret1, secret2];
        assert_eq!(
            init_str(secrets).unwrap(),
            format!("ZOUZOU:zozo/zaza{LINE_ENDING}ZONZON:zuzu/zaza{LINE_ENDING}")
        );
    }
}
