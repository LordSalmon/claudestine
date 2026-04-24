use std::{
    env::home_dir,
    path::{Path, PathBuf},
};

use crate::container::ignore::{IgnoreRule, IgnoreRuleSet};

pub struct VolumeMapping {
    source: Option<PathBuf>,
    destination: PathBuf,
}

impl VolumeMapping {
    pub fn new(source: PathBuf, destination: PathBuf) -> Self {
        Self { source: Some(source), destination }
    }

    pub fn serialize(&self) -> String {
        if let Some(ref source) = self.source {
            format!("{}:{}", source.display(), self.destination.display())
        } else {
            format!("{}", self.destination.display())
        }
    }
}

pub fn volume_mappings_by_ignore_rule_sets(
    ignore_rule_sets: Vec<IgnoreRuleSet>,
) -> Vec<VolumeMapping> {
    let mut mappings = Vec::new();
    if let Some(claude_config) = claude_config_dir_mapping() {
        mappings.push(claude_config);
    }
    if let Some(claude_config_file) = claude_config_file_mapping() {
        mappings.push(claude_config_file);
    }
    mappings.push(base_mapping());
    mappings.extend(
        ignore_rule_sets
            .iter()
            .flat_map(|rs| rs.rules.iter().flat_map(|r| mapping_by_ignore_rule(r))),
    );
    let real_destinations: std::collections::HashSet<PathBuf> = mappings
        .iter()
        .filter(|m| m.source.is_some() && m.source.as_deref() != Some(empty_file_mount().as_path()))
        .map(|m| m.destination.clone())
        .collect();
    mappings.retain(|m| {
        m.source.as_deref() != Some(empty_file_mount().as_path())
            || !real_destinations.contains(&m.destination)
    });
    mappings
}

fn mapping_by_ignore_rule(ignore_rule: &IgnoreRule) -> Vec<VolumeMapping> {
    ignore_rule
        .files
        .iter()
        .map(|f| {
            let local = Path::new(".").join(f.strip_prefix("/").unwrap_or(f));
            let source = if ignore_rule.exclude {
                Some(local)
            } else if local.is_file() {
                Some(empty_file_mount())
            } else {
                None
            };
            VolumeMapping {
                source,
                destination: workdir().join(f.strip_prefix("/").unwrap_or(f)),
            }
        })
        .collect()
}

fn base_mapping() -> VolumeMapping {
    VolumeMapping {
        source: Some(Path::new(".").to_path_buf()),
        destination: workdir(),
    }
}

fn claude_config_dir_mapping() -> Option<VolumeMapping> {
    let home_dir = home_dir();
    if let Some(home) = home_dir {
        Some(VolumeMapping {
            source: Some(home.join(".claude")),
            destination: container_user_dir().join(".claude"),
        })
    } else {
        None
    }
}

fn claude_config_file_mapping() -> Option<VolumeMapping> {
    if let Some(home_dir) = home_dir() {
        let path = home_dir.join(".claude.json");
        if path.exists() {
            Some(VolumeMapping {
                source: Some(path),
                destination: container_user_dir().join(".claude.json"),
            })
        } else {
            None
        }
    } else {
        None
    }
}

pub fn claudestine_config_mapping() -> VolumeMapping {
    VolumeMapping {
        source: None,
        destination: workdir().join(".claudestine"),
    }
}

fn container_user_dir() -> PathBuf {
    Path::new("/root").to_path_buf()
}

fn workdir() -> PathBuf {
    Path::new("/usr").join("src").join("claudestine")
}

fn empty_file_mount() -> PathBuf {
    PathBuf::from("/dev/null")
}
