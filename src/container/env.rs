#[cfg(target_os = "macos")]
use anyhow::{Result, bail};
#[cfg(target_os = "macos")]
use serde::{Deserialize, Serialize};
#[cfg(target_os = "macos")]
use std::process::Command;

pub enum HostEnvVariable {
    Value { value: String },
    Reference { name: String },
}

pub struct EnvRecord<'a> {
    pub name: &'a str,
    pub host: HostEnvVariable,
}

impl<'a> EnvRecord<'a> {
    pub fn serialize(&'a self) -> String {
        match &self.host {
            HostEnvVariable::Value { value } => {
                format!("{}={}", self.name, value)
            }
            HostEnvVariable::Reference { name } => {
                format!("{}=${}", self.name, name)
            }
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(target_os = "macos")]
struct ClaudeOAuth {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
    scopes: Vec<String>,
    subscription_type: String,
    rate_limit_tier: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(target_os = "macos")]
pub struct ClaudeCredentials {
    claude_ai_oauth: ClaudeOAuth,
}

#[cfg(target_os = "linux")]
pub fn security_token_env<'a>() -> Option<EnvRecord<'a>> {
    None
}

#[cfg(target_os = "macos")]
pub fn security_token_env<'a>() -> Option<EnvRecord<'a>> {
    if let Ok(Some(credentials)) = credentials_record() {
        Ok(Some(EnvRecord {
            name: "CLAUDE_CODE_OAUTH_TOKEN",
            host: HostEnvVariable::Value {
                value: parsed_oauth_credentials.claude_ai_oauth.access_token,
            },
        }))
    } else {
        None
    }
}

#[cfg(target_os = "macos")]
pub fn credentials_record() -> Result<Option<ClaudeCredentials>> {
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "Claude Code-credentials",
            "-w",
        ])
        .output()?;

    if !output.status.success() {
        bail!("Couldn't parse the command output")
    }

    let json = String::from_utf8(output.stdout)?;
    if let Ok(parsed_credentials) = serde_json::from_str::<ClaudeCredentials>(json.trim()) {
        Ok(Some(parsed_credentials))
    } else {
        Ok(None)
    }
}
