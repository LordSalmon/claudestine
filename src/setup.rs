#[cfg(target_os = "linux")]
use anyhow::Result;

#[cfg(target_os = "macos")]
use std::{env::home_dir, fs, path::PathBuf};

#[cfg(target_os = "macos")]
use anyhow::{Result, bail};

#[cfg(target_os = "macos")]
use log::info;

#[cfg(target_os = "macos")]
use crate::container::env::credentials_record;

#[cfg(target_os = "linux")]
pub fn setup() -> Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn setup() -> Result<()> {
    if let Ok(Some(keychain_credentials)) = credentials_record() {
        let credentials_path = claude_credentials_path()?;
        if credentials_path.exists() {
            info!("Credentials file already exists. Skipping...");
            Ok(())
        } else {
            fs::write(
                credentials_path,
                serde_json::to_string(&keychain_credentials).unwrap(),
            )?;
            Ok(())
        }
    } else {
        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub fn claude_credentials_path() -> Result<PathBuf> {
    if let Some(home_dir) = home_dir() {
        Ok(home_dir.join(".claude").join(".credentials.json"))
    } else {
        bail!("Couldn't find home dir")
    }
}
