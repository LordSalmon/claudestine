# Claudestine

Run your Claude Code in a docker environment that lets you shield sensitive information and files which Claude Code could access elsewhere.

## Installation

Run:

```bash
curl -fsSL https://raw.githubusercontent.com/LordSalmon/claudestine/refs/heads/main/scripts/install.sh | bash
```

And follow the instructions

## Setup

### On Macos (not necessary for Linux)

Run:

```zsh
claudestine setup
```

### Repo setup

Run:

```bash
claudestine init
```

This creates a new folder `.claudestine` in which you can find `config.toml`, `Dockerfile` and `isolates`

`config.toml` lets you specify more isolate files, but for most cases `isolates` should suffice

`Dockerfile` is the setup for the environment Claude Code will run in. In there you should:

- Install languages you are using for in process syntax check and linting
-

## Notes

- If you are on windows: Sad story :'-(

- Docker containers are not a full isolation since they share the same kernel. There were two other project [Nanoclaw Firecracker](https://github.com/zwchristie/nanoclaw-firecracker) and [FireCClaw](https://github.com/aawlbt/FireCClaw) using firecracker but unfortunately they disappeared :( The goal for this project was to provide acceptable security measures for Claude Code with minimal DX drawbacks.

- The project is not battle tested and not illustirous to bugs.

## System requirements

Claude code installed with a plan
