//! Module handling all the CLI arguments

use crate::consts::{TOOL_NAME, TOOL_VERSION};
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete_command::Shell;
use std::path::PathBuf;

macro_rules! env_var {
    ($name:literal) => {
        concat!("KEYCLI_", $name)
    };
}

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Parser)]
#[command(
    name = TOOL_NAME,
    version = TOOL_VERSION,
    about = "A env manager which stores your secrets in your OS keyring",
    long_about = None,
    styles = STYLES,
    after_help = "Examples:

# Create a .keycli.conf from a keycli.tpl and populate your keyring
keycli init

# Create a .keycli.conf from scratch and populate your keyring
keycli init -a my_app -s PASS -s PASS2 -s PASS3:another_app

# Run a shell with declared env vars
keycli shell

# Load env vars
eval $(keycli load) # Or keycli-load if you installed the alias

# Unload env vars
eval $(keycli unload) # Or keycli-unload if you installed the alias

# Save vars without .keycli.conf file
keycli save -a custom_app -s ZOZO -s ZAZA

# Load vars without .keycli.conf file
keycli load -a custom_app -s ZOZO -s ZAZA

# Install completions and aliases
keycli alias zsh >> ~/.zshrc
keycli completion zsh > ~/.zfunc/_keycli
keycli completion zsh keycli-load > ~/.zfunc/_keycli-load
keycli completion zsh keycli-unload > ~/.zfunc/_keycli-unload
",
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args)]
pub struct GlobalArgs {
    /// Turn on verbose output
    #[arg(short, long, global = true, env = env_var!("VERBOSE"))]
    pub verbose: bool,
}

#[derive(Args)]
pub struct CommonOptions {
    /// Extra secrets to load with the config file (if specified) on the form
    /// ENV_VAR ENV_VAR:appName/secret, ENV_VAR:secret, :appName/secret or :secret
    #[arg(short, long, env = env_var!("EXTRA_SECRETS"), conflicts_with = "secrets")]
    pub extra_secrets: Vec<String>,

    /// Secrets to use on the form
    /// ENV_VAR ENV_VAR:appName/secret, ENV_VAR:secret, :appName/secret or :secret
    #[arg(short, long, env = env_var!("SECRETS"), conflicts_with_all = ["extra_secrets", "config"])]
    pub secrets: Vec<String>,

    /// App name which needs the secret
    #[arg(short, long, env = env_var!("APP_NAME"))]
    pub app_name: Option<String>,
}

#[derive(Args)]
pub struct AllCommonOptions {
    #[command(flatten)]
    pub common: CommonOptions,

    /// Config file
    #[arg(short, long, env = env_var!("CONFIG"), conflicts_with = "secrets")]
    pub config: Option<PathBuf>,

    /// Check for changes in associated the tpl file
    #[arg(short = 't', long = "no-check-template", env = env_var!("CHECK_TEMPLATE"), default_value = "true", action = clap::ArgAction::SetFalse)]
    pub check_tpl: bool,
}

#[derive(Args)]
pub struct SaveArgs {
    /// Interactive prompt for passwords. If false, pull secrets from env
    #[arg(short, long = "no-interactive", env = env_var!("INTERACTIVE"), default_value = "true", action = clap::ArgAction::SetFalse)]
    pub interactive: bool,

    /// Force the overwrite of secrets in the keyring
    #[arg(short, long, env = env_var!("FORCE"), default_value = "false", action = clap::ArgAction::SetTrue)]
    pub force: bool,
}

#[derive(Args)]
pub struct LoadArgs {
    /// Overwrite existing env vars
    #[arg(short, long = "no-overwrite", env = env_var!("OVERWRITE"), default_value = "true", action = clap::ArgAction::SetFalse)]
    pub overwrite: bool,
}

#[derive(ValueEnum, Clone)]
pub enum AliasCommand {
    #[value(name = "keycli-load")]
    Load,

    #[value(name = "keycli-unload")]
    Unload,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Print sourcable shell script to load secrets to the environment. Used by keycli-load
    Load {
        #[command(flatten)]
        all_common: AllCommonOptions,

        #[command(flatten)]
        load: LoadArgs,
    },

    /// Print sourcable shell script to unload secrets from the environment. Used by keycli-unload
    Unload {
        #[command(flatten)]
        all_common: AllCommonOptions,
    },

    /// List all env vars managed by keycli with the current options and args
    List {
        #[command(flatten)]
        all_common: AllCommonOptions,
    },

    /// Save secrets to the keyring
    Save {
        #[command(flatten)]
        all_common: AllCommonOptions,

        #[command(flatten)]
        save: SaveArgs,
    },

    /// Clear the keyring
    Clear {
        #[command(flatten)]
        all_common: AllCommonOptions,

        /// Interactive prompt for validation of password deletion. If false, the deletion will be
        /// automatic
        #[arg(short, long = "no-interactive", env = env_var!("INTERACTIVE"), default_value = "true", action = clap::ArgAction::SetFalse)]
        interactive: bool,
    },

    /// Execute a command with env vars
    Exec {
        #[command(flatten)]
        all_common: AllCommonOptions,

        #[command(flatten)]
        load: LoadArgs,

        /// Binary to execute
        binary: String,

        /// Arguments of the binary to execute
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Execute a shell with env vars
    Shell {
        #[command(flatten)]
        all_common: AllCommonOptions,

        #[command(flatten)]
        load: LoadArgs,
    },

    /// Create a .keycli.conf from secrets and / or a keycli.tpl
    Init {
        #[command(flatten)]
        common: CommonOptions,

        #[command(flatten)]
        save: SaveArgs,

        /// Config file
        #[arg(short, long, env = env_var!("CONFIG"), default_value = ".keycli.conf")]
        config: PathBuf,

        /// Template file
        #[arg(short, long, env = env_var!("TEMPLATE"), default_value = "keycli.tpl")]
        template: PathBuf,
    },

    /// Generate shell aliases
    Alias {
        /// The shell to generate the aliases for
        shell: Shell,
    },

    /// Generate shell completion scripts
    Completion {
        /// The shell to generate the completion script for
        shell: Shell,

        #[arg(value_enum)]
        alias: Option<AliasCommand>,
    },
}
