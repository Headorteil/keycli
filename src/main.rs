//! Main module, declares the cli

pub mod cli;
pub mod commands;
pub mod completion;
pub mod config;
pub mod consts;
pub mod secrets;

use crate::cli::Cli;
use crate::cli::{AliasCommand, Commands};
use crate::completion::{ALIASES, BASH_LOAD, BASH_UNLOAD, ZSH_LOAD, ZSH_UNLOAD};
use crate::consts::LINE_ENDING;
use anyhow::{Context, Result, anyhow};
use clap::CommandFactory;
use clap::Parser;
use clap::builder::styling::AnsiColor;
use clap_complete_command::Shell;
use dialoguer::Confirm;
use env_logger::fmt::style::Style;
use std::io::Write;
use std::{env, fs};

/// Parse command line options, initialize the logger and run the real main
pub fn main() {
    let cli = cli::Cli::parse();
    let level = {
        if cli.global.verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        }
    };

    env_logger::Builder::new()
        .filter_level(level)
        .format(|buf, record| {
            let level = record.level();
            let color = match level {
                log::Level::Info => AnsiColor::BrightBlue,
                log::Level::Error => AnsiColor::Red,
                log::Level::Warn => AnsiColor::Yellow,
                log::Level::Debug => AnsiColor::Blue,
                log::Level::Trace => AnsiColor::Cyan,
            };
            let style = Style::new().fg_color(Some(color.into())).bold();
            write!(
                buf,
                "{}{level}{} {}{LINE_ENDING}",
                style.render(),
                style.render_reset(),
                record.args()
            )
        })
        .init();

    if cli.global.verbose {
        log::debug!("Verbose mode enabled");
    }

    if let Err(e) = run(cli) {
        log::error!("{e}");
        std::process::exit(1);
    }
}

/// Runs the right subcommand
pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Load { all_common, load } => {
            let parsed_secrets = secrets::parse_secrets(
                all_common.common.app_name,
                all_common.common.extra_secrets,
                all_common.config,
                all_common.common.secrets,
                load.overwrite,
                all_common.check_tpl,
                true,
            )?;
            let env_content = commands::load(parsed_secrets)?;
            print!("{env_content}");
            Ok(())
        }
        Commands::Unload { all_common } => {
            let parsed_secrets = secrets::parse_secrets(
                all_common.common.app_name,
                all_common.common.extra_secrets,
                all_common.config,
                all_common.common.secrets,
                true,
                all_common.check_tpl,
                true,
            )?;
            let env_content = commands::unload(parsed_secrets)?;
            print!("{env_content}");
            Ok(())
        }
        Commands::Save { all_common, save } => {
            let parsed_secrets = secrets::parse_secrets(
                all_common.common.app_name.clone(),
                all_common.common.extra_secrets,
                all_common.config,
                all_common.common.secrets,
                true,
                all_common.check_tpl,
                true,
            )?;
            let is_interractive = save.interactive && !save.force;
            commands::save(
                all_common.common.app_name,
                parsed_secrets,
                is_interractive,
                save.force,
            )?;
            Ok(())
        }
        Commands::Clear {
            all_common,
            interactive,
        } => {
            let parsed_secrets = secrets::parse_secrets(
                all_common.common.app_name.clone(),
                all_common.common.extra_secrets,
                all_common.config,
                all_common.common.secrets,
                true,
                all_common.check_tpl,
                true,
            )?;
            commands::clear(parsed_secrets, interactive)
        }
        Commands::Exec {
            all_common,
            load,
            binary,
            args,
        } => {
            let parsed_secrets = secrets::parse_secrets(
                all_common.common.app_name,
                all_common.common.extra_secrets,
                all_common.config,
                all_common.common.secrets,
                load.overwrite,
                all_common.check_tpl,
                true,
            )?;
            commands::exec(parsed_secrets, &binary, args)
        }
        Commands::Shell { all_common, load } => {
            let parsed_secrets = secrets::parse_secrets(
                all_common.common.app_name,
                all_common.common.extra_secrets,
                all_common.config,
                all_common.common.secrets,
                load.overwrite,
                all_common.check_tpl,
                true,
            )?;
            let shell = env::var("SHELL").context("SHELL environment variable not found")?;
            commands::exec(parsed_secrets, &shell, Vec::new())
        }
        Commands::Init {
            common,
            config,
            template,
            save,
        } => {
            let template_opt = template.exists().then_some(template);
            let is_interractive = save.interactive && !save.force;
            if config.exists() {
                if is_interractive
                    && Confirm::new()
                        .with_prompt(format!("{} already exist, overwrite it?", config.display()))
                        .default(false)
                        .interact()?
                {
                    println!("Old config file:");
                    println!();
                    println!("{}", fs::read_to_string(config.clone())?);
                } else if !save.force {
                    if !is_interractive {
                        log::warn!(
                            "{} already exist and force is not specified, aborting",
                            config.display()
                        );
                    }
                    return Ok(());
                }
            }
            let parsed_secrets = secrets::parse_secrets(
                common.app_name.clone(),
                common.extra_secrets,
                template_opt.clone(),
                common.secrets,
                true,
                false,
                false,
            )?;
            let new_secrets =
                commands::save(common.app_name, parsed_secrets, is_interractive, save.force)?;
            let content = commands::init(new_secrets, template_opt)?;
            fs::write(&config, content)
                .with_context(|| format!("Failed to write file: {}", config.display()))
        }
        Commands::Alias { shell } => {
            match shell {
                Shell::Zsh | Shell::Bash => {
                    println!("{}", ALIASES.replace('\n', LINE_ENDING))
                }
                _ => return Err(anyhow!("Shell not yet suported")),
            }
            Ok(())
        }
        Commands::Completion { shell, alias } => {
            match (shell, alias) {
                (Shell::Zsh, Some(AliasCommand::Load)) => {
                    println!("{}", ZSH_LOAD.replace('\n', LINE_ENDING))
                }
                (Shell::Bash, Some(AliasCommand::Load)) => {
                    println!("{}", BASH_LOAD.replace('\n', LINE_ENDING))
                }
                (Shell::Zsh, Some(AliasCommand::Unload)) => {
                    println!("{}", ZSH_UNLOAD.replace('\n', LINE_ENDING))
                }
                (Shell::Bash, Some(AliasCommand::Unload)) => {
                    println!("{}", BASH_UNLOAD.replace('\n', LINE_ENDING))
                }
                (_, None) => shell.generate(&mut Cli::command(), &mut std::io::stdout()),
                _ => return Err(anyhow!("Shell not yet suported")),
            };
            Ok(())
        }
    }
}
