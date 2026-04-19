use std::{
    fs,
    path::{Path, PathBuf},
};

use glob::glob;
use log::info;

pub struct IgnoreRule {
    pub files: Vec<PathBuf>,
    pub exclude: bool,
}

pub struct IgnoreRuleSet {
    pub rules: Vec<IgnoreRule>,
}

pub fn parse_ignore_rule_set(ignore_file_path: PathBuf) -> IgnoreRuleSet {
    info!("Capturing ignore rules from {}", ignore_file_path.display());
    let content = fs::read_to_string(ignore_file_path).unwrap();
    return IgnoreRuleSet {
        rules: sanitize_ignore_file(content)
            .iter()
            .map(|l| IgnoreRule {
                files: parse_ignore_rule(l),
                exclude: l.starts_with("!"),
            })
            .collect(),
    };
}

fn sanitize_ignore_file(content: String) -> Vec<String> {
    let lines = Vec::from_iter(
        content
            .split("\n")
            .map(|l| l.split("#").nth(0).unwrap().trim())
            .filter(|l| l.len() > 0)
            .map(|l| l.to_string())
            .map(|l| {
                if let Some(stripped) = l.strip_prefix("/") {
                    stripped.to_string()
                } else {
                    l
                }
            })
            .map(|l| {
                if l.starts_with("!/") {
                    format!("!{}", l.strip_prefix("!/").unwrap()).to_string()
                } else {
                    l
                }
            })
            .into_iter(),
    );
    return lines;
}

fn parse_ignore_rule(line: &String) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if line.contains("*") {
        for path in glob(line.as_str()).expect("Couldn't read file") {
            match path {
                Ok(p) => paths.push(p),
                Err(_) => paths.push(Path::new(line.replace("!", "").as_str()).to_path_buf()),
            };
        }
    } else {
        paths.push(Path::new(line.replace("!", "").as_str()).to_path_buf())
    }
    paths
}
