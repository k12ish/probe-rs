use crate::DebuggerError;
use anyhow::{anyhow, Result};
use probe_rs::{DebugProbeSelector, WireProtocol};
use probe_rs_cli_util::rtt;
use serde::Deserialize;
use std::{env::current_dir, path::PathBuf, str::FromStr};

/// Shared options for all session level configuration.
#[derive(Clone, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfig {
    /// Path to the requested working directory for the debugger
    pub(crate) cwd: Option<PathBuf>,

    /// Binary to debug as a path. Relative to `cwd`, or fully qualified.
    pub(crate) program_binary: Option<PathBuf>,

    /// CMSIS-SVD file for the target. Relative to `cwd`, or fully qualified.
    pub(crate) svd_file: Option<PathBuf>,

    /// The number associated with the debug probe to use. Use 'list' command to see available probes
    #[serde(alias = "probe")]
    pub(crate) probe_selector: Option<DebugProbeSelector>,

    /// The MCU Core to debug. Default is 0
    #[serde(default)]
    pub(crate) core_index: usize,

    /// The target to be selected.
    pub(crate) chip: Option<String>,

    /// Protocol to use for target connection
    #[serde(rename = "wire_protocol")]
    pub(crate) protocol: Option<WireProtocol>,

    /// Protocol speed in kHz
    pub(crate) speed: Option<u32>,

    /// Assert target's reset during connect
    #[serde(default)]
    pub(crate) connect_under_reset: bool,

    /// Allow the chip to be fully erased
    #[serde(default)]
    pub(crate) allow_erase_all: bool,

    /// IP port number to listen for incoming DAP connections, e.g. "50000"
    pub(crate) port: Option<u16>,

    /// Flash the target before debugging
    #[serde(default)]
    pub(crate) flashing_enabled: bool,

    /// Reset the target after flashing
    #[serde(default)]
    pub(crate) reset_after_flashing: bool,

    /// Halt the target after reset
    #[serde(default)]
    pub(crate) halt_after_reset: bool,

    /// Do a full chip erase, versus page-by-page erase
    #[serde(default)]
    pub(crate) full_chip_erase: bool,

    /// Restore erased bytes that will not be rewritten from ELF
    #[serde(default)]
    pub(crate) restore_unwritten_bytes: bool,

    /// Level of information to be logged to the debugger console (Error, Info or Debug )
    #[serde(default = "default_console_log")]
    pub(crate) console_log_level: Option<ConsoleLog>,

    #[serde(flatten)]
    pub(crate) rtt: rtt::RttConfig,
}

impl SessionConfig {
    /// Validate the new cwd, or else set it from the environment.
    pub(crate) fn validate_and_update_cwd(&mut self, new_cwd: Option<PathBuf>) {
        self.cwd = match new_cwd {
            Some(temp_path) => {
                if temp_path.is_dir() {
                    Some(temp_path)
                } else if let Ok(current_dir) = current_dir() {
                    Some(current_dir)
                } else {
                    log::error!("Cannot use current working directory. Please check existence and permissions.");
                    None
                }
            }
            None => {
                if let Ok(current_dir) = current_dir() {
                    Some(current_dir)
                } else {
                    log::error!("Cannot use current working directory. Please check existence and permissions.");
                    None
                }
            }
        };
    }

    /// If the path to the program to be debugged is relative, we join if with the cwd.
    pub(crate) fn qualify_and_update_os_file_path(
        &mut self,
        os_file_to_validate: Option<PathBuf>,
    ) -> Result<PathBuf, DebuggerError> {
        match os_file_to_validate {
            Some(temp_path) => {
                let mut new_path = PathBuf::new();
                if temp_path.is_relative() {
                    if let Some(cwd_path) = self.cwd.clone() {
                        new_path.push(cwd_path);
                    } else {
                        return Err(DebuggerError::Other(anyhow!(
                            "Invalid value {:?} for `cwd`",
                            self.cwd
                        )));
                    }
                }
                new_path.push(temp_path);
                Ok(new_path)
            }
            None => Err(DebuggerError::Other(anyhow!("Missing value for file."))),
        }
    }
}

fn default_console_log() -> Option<ConsoleLog> {
    Some(ConsoleLog::Error)
}

/// The level of information to be logged to the debugger console. The DAP Client will set appropriate RUST_LOG env for 'launch' configurations,  and will pass the rust log output to the client debug console.
#[derive(Copy, Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum ConsoleLog {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::str::FromStr for ConsoleLog {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_ascii_lowercase()[..] {
            "error" => Ok(ConsoleLog::Error),
            "warn" => Ok(ConsoleLog::Error),
            "info" => Ok(ConsoleLog::Info),
            "debug" => Ok(ConsoleLog::Debug),
            "trace" => Ok(ConsoleLog::Trace),
            _ => Err(format!(
                "'{}' is not a valid console log level. Choose from [error, warn, info, debug, or trace].",
                s
            )),
        }
    }
}
