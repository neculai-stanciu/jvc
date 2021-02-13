#![cfg(windows)]
use std::process::Command;

use crate::config::JvcConfig;

use super::executor::Executor;
use crate::commands::env::make_symlink;
use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use log::{debug, info, warn};
use structopt::StructOpt;
use winreg::{
    enums::{RegType, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE},
    RegKey, RegValue,
};

#[derive(Debug, StructOpt)]
pub struct Setup {}

impl Setup {
    pub async fn set_persistent_env_var(&self, name: &str, value: &str) {
        let permanent_set = format!("SETX {} {}", name, value);

        debug!("running command: {} on cmd", permanent_set);
        Command::new("cmd")
            .args(&["/C", permanent_set.as_ref()])
            .output()
            .or_else(|e| {
                debug!("run into error on command execution: {:#?}", e);
                Err(e)
            })
            .ok();
    }

    pub async fn set_persistent_path(&self, value: &str) -> Result<()> {
        println!("Extend path with value: {}", value);
        let root = RegKey::predef(HKEY_CURRENT_USER);
        let current_path_value = get_windows_path_var()?;

        if let Some(path_val) = current_path_value {
            if path_val.contains(value) {
                info!("Ignore adding value because is already configured");
                return Ok(());
            }
            debug!("Path values: \n {}", path_val);
            let final_path = format!(
                "{}{}installation{}bin;{}",
                value,
                std::path::MAIN_SEPARATOR,
                std::path::MAIN_SEPARATOR,
                path_val
            );
            debug!("End value: \n {}", final_path);
            let environment = root.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

            let reg_value = RegValue {
                bytes: string_to_winreg_bytes(final_path.as_ref()),
                vtype: RegType::REG_EXPAND_SZ,
            };
            environment.set_raw_value("PATH", &reg_value)?;
        }

        Ok(())
    }
}

#[async_trait]
impl Executor for Setup {
    async fn execute(self, config: JvcConfig) -> Result<()> {
        let shell_path = make_symlink(&config);
        let _bin_path = shell_path.join("installation").join("bin");

        let shell_path_as_str = shell_path
            .to_str()
            .ok_or(anyhow!("Cannot set shell path."))?;

        debug!("Create a JVC_SHELL_PATH {}", shell_path_as_str);

        let base_dir = config.get_base_dir_or_default();
        let jvc_dir_as_str = base_dir
            .to_str()
            .ok_or(anyhow!("Cannot read base directory."))?;

        self.set_persistent_env_var("JVC_SHELL_PATH", shell_path_as_str)
            .await;
        self.set_persistent_env_var("JVC_DIR", jvc_dir_as_str).await;
        self.set_persistent_env_var("JVC_LOGLEVEL", config.log_level.into())
            .await;
        self.set_persistent_env_var("JVC_PROVIDER", &config.java_provider.as_str())
            .await;

        let path_value = format!("%JVC_SHELL_PATH%");
        let _result = self.set_persistent_path(path_value.as_ref()).await?;

        Ok(())
    }
}

/// Encodes a utf-8 string as a null-terminated UCS-2 string in bytes
#[cfg(windows)]
pub fn string_to_winreg_bytes(s: &str) -> Vec<u8> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    let v: Vec<u16> = OsStr::new(s).encode_wide().chain(Some(0)).collect();
    unsafe { std::slice::from_raw_parts(v.as_ptr().cast::<u8>(), v.len() * 2).to_vec() }
}

// This is used to decode the value of HKCU\Environment\PATH. If that
// key is not unicode (or not REG_SZ | REG_EXPAND_SZ) then this
// returns null.  The winreg library itself does a lossy unicode
// conversion.
#[cfg(windows)]
pub fn string_from_winreg_value(val: &winreg::RegValue) -> Option<String> {
    use std::slice;

    match val.vtype {
        RegType::REG_SZ | RegType::REG_EXPAND_SZ => {
            // Copied from winreg
            let words = unsafe {
                #[allow(clippy::cast_ptr_alignment)]
                slice::from_raw_parts(val.bytes.as_ptr().cast::<u16>(), val.bytes.len() / 2)
            };
            String::from_utf16(words).ok().map(|mut s| {
                while s.ends_with('\u{0}') {
                    s.pop();
                }
                s
            })
        }
        _ => None,
    }
}

// Get the windows PATH variable out of the registry as a String. If
// this returns None then the PATH variable is not unicode and we
// should not mess with it.
fn get_windows_path_var() -> Result<Option<String>> {
    use std::io;
    let root = RegKey::predef(HKEY_CURRENT_USER);
    let environment = root.open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)?;

    let reg_value = environment.get_raw_value("PATH");
    match reg_value {
        Ok(val) => {
            if let Some(s) = string_from_winreg_value(&val) {
                Ok(Some(s))
            } else {
                warn!("the registry key HKEY_CURRENT_USER\\Environment\\PATH does not contain valid Unicode. \
                    Not modifying the PATH variable");
                Ok(None)
            }
        }
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(Some(String::new())),
        Err(_e) => Err(anyhow!("Cannot read path")),
    }
}
