use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead};
use nix::sys::statvfs::statvfs;
use serde::{Serialize, Deserialize};
use thiserror::Error;

const SECTOR_SIZE: u64 = 512;
const VIRTUAL_FS: [&str; 6] = [
    "sysfs", "proc", "tmpfs", "devtmpfs", "cgroup2", "pstore"
];

#[derive(Debug, Error)]
pub enum ScoutError {
    #[error("Failed to read file: {0}")]
    IOError(#[from] std::io::Error),
    
    #[error("Failed to parse integer: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoStats {
    pub reads_completed: u64,
    pub sectors_read: u64,
    pub writes_completed: u64,
    pub sectors_written: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountUsage {
    pub mount_point: String,
    pub total_space: u64,
    pub used_space: u64,
    pub free_space: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub name: String,
    pub capacity: Option<u64>,
    pub io_stats: Option<IoStats>,
    pub mount_usage: Option<MountUsage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub capacity: Option<u64>,
    pub io_stats: Option<IoStats>,
    pub partitions: Vec<PartitionInfo>,
}

/// Reads disk capacity from `/sys/class/block/{device}/size`
pub fn read_capacity(device: &str) -> Option<u64> {
    let sys_path = format!("/sys/class/block/{}/size", device);
    fs::read_to_string(sys_path)
        .ok()?
        .trim()
        .parse::<u64>()
        .map(|sectors| sectors * SECTOR_SIZE)
        .ok()
}

/// Parses `/proc/diskstats` to extract I/O statistics
pub fn parse_diskstats_raw() -> Result<HashMap<String, IoStats>, ScoutError> {
    let file = fs::File::open("/proc/diskstats")?;
    let reader = io::BufReader::new(file);
    let mut stats_map = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 14 || parts[2].starts_with("loop") {
            continue;
        }

        let io_stats = IoStats {
            reads_completed: parts[3].parse().unwrap_or(0),
            sectors_read: parts[5].parse().unwrap_or(0),
            writes_completed: parts[7].parse().unwrap_or(0),
            sectors_written: parts[9].parse().unwrap_or(0),
        };
        stats_map.insert(parts[2].to_string(), io_stats);
    }
    Ok(stats_map)
}

/// Parses `/proc/mounts` to get disk usage statistics
pub fn parse_mounts_usage() -> Result<HashMap<String, MountUsage>, ScoutError> {
    let file = fs::File::open("/proc/mounts")?;
    let reader = io::BufReader::new(file);
    let mut mount_map = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 || parts[0].starts_with("/dev/loop") || VIRTUAL_FS.contains(&parts[2]) {
            continue;
        }

        if let Ok(vfs) = statvfs(parts[1]) {
            let block_size = vfs.fragment_size() as u64;
            let total = vfs.blocks() * block_size;
            let available = vfs.blocks_available() * block_size;
            let used = total - (vfs.blocks_free() * block_size);

            mount_map.insert(
                parts[0].to_string(),
                MountUsage {
                    mount_point: parts[1].to_string(),
                    total_space: total,
                    used_space: used,
                    free_space: available,
                },
            );
        }
    }
    Ok(mount_map)
}

/// Determines if a device is a partition
pub fn is_partition(device: &str) -> bool {
    if device.starts_with("loop") {
        return false;
    }
    if device.starts_with("nvme") {
        return device.rsplit('p').next().and_then(|s| s.parse::<u32>().ok()).is_some();
    }
    device.len() > 3 && device.chars().last().unwrap().is_ascii_digit()
}

/// Extracts the parent disk name from a partition
pub fn parent_disk(device: &str) -> String {
    if device.starts_with("nvme") {
        if let Some(pos) = device.rfind('p') {
            return device[..pos].to_string();
        }
    }
    device.trim_end_matches(|c: char| c.is_ascii_digit()).to_string()
}

/// Parses all disk information and returns it as a HashMap
pub fn get_disks() -> Result<HashMap<String, DiskInfo>, ScoutError> {
    let diskstats = parse_diskstats_raw()?;
    let mount_usage = parse_mounts_usage()?;
    let mut disks = HashMap::new();

    for (device, io_stats) in diskstats {
        if is_partition(&device) {
            let parent = parent_disk(&device);
            disks.entry(parent.clone())
                .or_insert_with(|| DiskInfo {
                    name: parent.clone(),
                    capacity: read_capacity(&parent),
                    io_stats: None,
                    partitions: Vec::new(),
                })
                .partitions.push(PartitionInfo {
                    name: device.clone(),
                    capacity: read_capacity(&device),
                    io_stats: Some(io_stats),
                    mount_usage: mount_usage.get(&format!("/dev/{}", device)).cloned(),
                });
        } else {
            disks.insert(
                device.clone(),
                DiskInfo {
                    name: device.clone(),
                    capacity: read_capacity(&device),
                    io_stats: Some(io_stats),
                    partitions: Vec::new(),
                },
            );
        }
    }
    Ok(disks)
}