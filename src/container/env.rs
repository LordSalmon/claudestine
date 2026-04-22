use anyhow::Result;
use serde::Deserialize;
use std::process::Command;

enum HostEnvVariable {
    Value { value: String },
    Reference { name: String },
}

pub struct EnvRecord<'a> {
    name: &'a str,
    host: HostEnvVariable,
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

#[derive(Deserialize)]
struct ClaudeOAuth {
    access_token: String,
    refresh_token: String,
    expires_at: i64,
    scopes: Vec<String>,
    subscription_type: String,
    rate_limit_tier: String,
}

#[derive(Deserialize)]
struct MacosClaudeSecret {
    claude_ai_oauth: ClaudeOAuth,
}

#[cfg(target_os = "linux")]
pub fn security_token_env<'a>() -> Result<Option<EnvRecord<'a>>> {
    Ok(None)
}

#[cfg(target_os = "macos")]
pub fn security_token_env<'a>() -> Result<Option<EnvRecord<'a>>> {
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "\"Claude Code-credentials\"",
            "-w",
        ])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let json = String::from_utf8(output.stdout)?;
    let parsed_oauth_credentials: MacosClaudeSecret = serde_json::from_str(json.as_str())?;

    Ok(Some(EnvRecord {
        name: "ANTHROPIC_API_KEY",
        host: HostEnvVariable::Reference {
            name: parsed_oauth_credentials.claude_ai_oauth.access_token,
        },
    }))
}
