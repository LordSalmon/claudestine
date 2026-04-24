use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
};

use anyhow::{Result, anyhow};
use log::{error, info, warn};
use serde::Deserialize;

use crate::config::BrokerCommandDef;

pub const CLIENT_SCRIPT: &str = include_str!("../assets/broker");

#[derive(Deserialize)]
struct BrokerRequest {
    command: String,
    args: HashMap<String, String>,
}

pub struct BrokerServer {
    commands: Vec<BrokerCommandDef>,
}

impl BrokerServer {
    pub fn new(commands: Vec<BrokerCommandDef>) -> Self {
        Self { commands }
    }

    /// Binds to an ephemeral port on 0.0.0.0, then spawns a listener thread.
    /// Returns a receiver that yields the chosen port once the socket is ready.
    pub fn start(self) -> Result<mpsc::Receiver<u16>> {
        let listener = TcpListener::bind("0.0.0.0:0")?;
        let port = listener.local_addr()?.port();

        let (tx, rx) = mpsc::channel::<u16>();

        thread::spawn(move || {
            tx.send(port).ok();
            info!("Broker: listening on 0.0.0.0:{}", port);
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        let commands = self.commands.clone();
                        thread::spawn(move || handle_connection(stream, commands));
                    }
                    Err(e) => {
                        error!("Broker: accept error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}

// ---------------------------------------------------------------------------
// Connection handler
// ---------------------------------------------------------------------------

fn handle_connection(stream: TcpStream, commands: Vec<BrokerCommandDef>) {
    let mut writer = match stream.try_clone() {
        Ok(w) => w,
        Err(e) => { error!("Broker: clone stream: {}", e); return; }
    };
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    if reader.read_line(&mut line).is_err() {
        return;
    }

    let (exit_code, stdout) = match serde_json::from_str::<BrokerRequest>(line.trim()) {
        Err(e) => (1, format!("broker: invalid request: {e}\n")),
        Ok(req) => {
            if req.command == "__list__" {
                list_commands(&commands)
            } else {
                execute(&req, &commands)
            }
        }
    };

    let header = format!("exit_code:{exit_code}\n");
    let _ = writer.write_all(header.as_bytes());
    let _ = writer.write_all(stdout.as_bytes());
}

// ---------------------------------------------------------------------------
// Command execution
// ---------------------------------------------------------------------------

fn execute(req: &BrokerRequest, commands: &[BrokerCommandDef]) -> (i32, String) {
    let Some(cmd) = commands.iter().find(|c| c.name == req.command) else {
        return (1, format!("broker: unknown command '{}'\n", req.command));
    };

    let mut resolved: HashMap<String, String> = HashMap::new();
    for arg in &cmd.args {
        if let Some(val) = req.args.get(&arg.name) {
            resolved.insert(arg.name.clone(), val.clone());
        } else if let Some(ref default) = arg.default {
            resolved.insert(arg.name.clone(), default.clone());
        } else if arg.required {
            return (1, format!("broker: missing required argument '{}'\n", arg.name));
        }
    }

    let mut invoke = cmd.invoke.clone();
    for (k, v) in &resolved {
        invoke = invoke.replace(&format!("${{{k}}}"), &shell_escape(v));
    }

    info!("Broker: {}", invoke);

    match std::process::Command::new("sh").args(["-c", &invoke]).output() {
        Err(e) => (1, format!("broker: failed to execute: {e}\n")),
        Ok(output) => {
            let code = output.status.code().unwrap_or(1);
            let mut out = String::from_utf8_lossy(&output.stdout).into_owned();
            if !output.stderr.is_empty() {
                out.push_str(&String::from_utf8_lossy(&output.stderr));
            }
            (code, out)
        }
    }
}

fn list_commands(commands: &[BrokerCommandDef]) -> (i32, String) {
    if commands.is_empty() {
        return (0, "No broker commands defined.\n".to_string());
    }
    let mut out = String::new();
    for cmd in commands {
        let desc = cmd.description.as_deref().unwrap_or("(no description)");
        out.push_str(&format!("{}: {}\n", cmd.name, desc));
        for arg in &cmd.args {
            let req = if arg.required { "required" } else { "optional" };
            let def = arg
                .default
                .as_deref()
                .map(|d| format!(", default={d}"))
                .unwrap_or_default();
            out.push_str(&format!("  {} ({}{def})\n", arg.name, req));
        }
    }
    (0, out)
}

// ---------------------------------------------------------------------------
// Shell escaping — single-quote wrap with embedded ' → '\''
// ---------------------------------------------------------------------------

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}
