use std::{
    env::home_dir,
    path::{Path, PathBuf},
};

use crate::container::{
    self,
    ignore::{IgnoreRule, IgnoreRuleSet},
};

pub struct VolumeMapping {
    source: Option<PathBuf>,
    destination: PathBuf,
}

impl VolumeMapping {
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
    // mappings.iter().filter(|mapping| {
    //     let duplicate_mappings = mappings.iter().filter(|m| {
    //         if let Some(source) = &m.source {
    //             &source.to_str() == &mapping.destination.to_str()
    //         } else {
    //             false
    //         }
    //     });
    //     // todo, remove when there is another mapping with the same destination and the current destination is /dev/null.
    // });
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

fn container_user_dir() -> PathBuf {
    Path::new("/root").to_path_buf()
}

fn workdir() -> PathBuf {
    Path::new("/usr").join("src").join("claudestine")
}

fn empty_file_mount() -> PathBuf {
    PathBuf::from("/dev/null")
}
