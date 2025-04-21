use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Holds system memory and VM tunable statistics, with defaults on error.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemoryInfo {
    pub total: u64,
    pub free: u64,
    pub available: u64,
    pub buffers: u64,
    pub cached: u64,
    pub swap_total: u64,
    pub swap_free: u64,
    pub anon_pages: u64,
    pub kernel_stack: u64,
    pub hugepage_size: u64,
    pub dirty_ratio: u64,
    pub dirty_background_ratio: u64,
    pub max_map_count: u64,
    pub overcommit_ratio: u64,
    pub swappiness: u64,
    pub nr_hugepages: u64,
    pub transparent_hugepages: bool,
}

impl MemoryInfo {
    /// Reads all memory info and VM tunables,
    /// substituting defaults if any read fails.
    pub fn new() -> Self {
        let m = parse_meminfo("/proc/meminfo");
        MemoryInfo {
            total: get_kb_default(&m, "MemTotal"),
            free: get_kb_default(&m, "MemFree"),
            available: get_kb_default(&m, "MemAvailable"),
            buffers: get_kb_default(&m, "Buffers"),
            cached: get_kb_default(&m, "Cached"),
            swap_total:get_kb_default(&m, "SwapTotal"),
            swap_free: get_kb_default(&m, "SwapFree"),
            anon_pages: get_kb_default(&m, "AnonPages"),
            kernel_stack: get_kb_default(&m, "KernelStack"),
            hugepage_size: get_kb_default(&m, "Hugepagesize"),    
            dirty_ratio: read_u64_default("/proc/sys/vm/dirty_ratio", 0),
            dirty_background_ratio: 
                read_u64_default("/proc/sys/vm/dirty_background_ratio", 0),

            max_map_count: read_u64_default("/proc/sys/vm/max_map_count", 0),
            overcommit_ratio: 
                read_u64_default("/proc/sys/vm/overcommit_ratio", 0),

            swappiness: read_u64_default("/proc/sys/vm/swappiness", 0),
            nr_hugepages: read_u64_default("/proc/sys/vm/nr_hugepages", 0),
            transparent_hugepages: read_transparent_hugepages_default(),
        }
    }
}

/// Parse `/proc/meminfo` into key → bytes, skipping unreadable lines.
fn parse_meminfo(path: &str) -> HashMap<String,u64> {
    let mut map = HashMap::new();
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if let Some((k, v)) = parse_line(&line) {
                map.insert(k, v);
            }
        }
    }
    map
}

/// Parse a “Key: <value> kB” line into (Key, value_in_bytes).
fn parse_line(line: &str) -> Option<(String,u64)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 && parts[0].ends_with(':') && parts[2] == "kB" {
        let key = parts[0].trim_end_matches(':').to_string();
        if let Ok(kb) = parts[1].parse::<u64>() {
            return Some((key, kb * 1024));
        }
    }
    None
}

/// Lookup a key in the meminfo map, defaulting to zero.
fn get_kb_default(map: &HashMap<String,u64>, key: &str) -> u64 {
    *map.get(key).unwrap_or(&0)
}

/// Read a one‑line file as u64, returning `default` on any error.
fn read_u64_default(path: &str, default: u64) -> u64 {
    fs::read_to_string(path)
        .map(|s| s.trim().parse::<u64>().unwrap_or(default))
        .unwrap_or(default)
}

/// Return whether transparent hugepages are enabled, defaulting to false.
fn read_transparent_hugepages_default() -> bool {
    let path = "/sys/kernel/mm/transparent_hugepage/enabled";
    if let Ok(s) = fs::read_to_string(path) {
        if let Some(start) = s.find('[') {
            if let Some(end) = s[start+1..].find(']') {
                let mode = &s[start+1..start+1+end];
                return mode != "never";
            }
        }
        return s.contains("always") || s.contains("madvise");
    }
    false
}
