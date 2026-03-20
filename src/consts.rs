//! Module containing diverse constants

pub const TOOL_NAME: &str = env!("CARGO_PKG_NAME");
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_CONF_FILE: &str = ".keycli.conf";
pub const PATH_HEADER: &str = "# keycli-template-path ";
pub const SHA_HEADER: &str = "# keycli-template-sha ";

#[cfg(windows)]
pub const LINE_ENDING: &str = "\r\n";
#[cfg(not(windows))]
pub const LINE_ENDING: &str = "\n";
