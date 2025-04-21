use std::fs::File;
use std::io::{BufRead, BufReader};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mount {
  pub source: String,
  pub target: String,
  pub fs_type: String,
  pub options: Vec<String>,
  pub dump: u32,
  pub pass: u32,
}

impl Mount {
  /// Parse one line of `/proc/self/mounts`.
  pub fn new(line: &str) -> Option<Self> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
      return None;
    }
    let source    = parts[0].to_string();
    let target    = parts[1].to_string();
    let fs_type   = parts[2].to_string();
    let options   = parts[3].split(',').map(ToString::to_string).collect();
    let dump      = parts[4].parse::<u32>().unwrap_or(0);
    let pass      = parts[5].parse::<u32>().unwrap_or(0);

    Some(Mount { source, target, fs_type, options, dump, pass })
  }
}

/// Returns all mounts by reading and parsing `/proc/self/mounts`.
pub fn get_mounts() -> Vec<Mount> {
  parse_mounts("/proc/self/mounts")
}

fn parse_mounts(path: &str) -> Vec<Mount> {
  let file = File::open(path);
  let file = match file {
    Ok(f)   => f,
    Err(_)  => return Vec::new(),
  };

  let reader = BufReader::new(file);
  reader
    .lines()
    .filter_map(|line| line.ok().and_then(|s| Mount::new(&s)))
    .collect()
}
