use std::collections::HashMap;
use std::fs::read_to_string; 
use std::io::{self, Error, ErrorKind};
use std::process::Command;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize,  Default)]
pub enum Architecture {
    AMD64,
    ARM64,
    RISCV,
    PPC64,
    S390X,
    SPARC64,
    I386,
    I686,
    ARM32,

    #[default]
    Unknown
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,

    #[default]
    Unknown
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cpu {
    architecture: Architecture,
    vendor_id: String,
    model: String,
    cpu_family: u32,
    op_modes: Vec<String>,
    address_size: u32,
    byte_order: ByteOrder,
    cores: u32,
    threads_per_core: u32,
    cores_per_socket: u32,
    sockets: u32,
    stepping: u32,
    frequency_boost_enabled: bool,
    cpu_scaling_pct: f32,
    cpu_max_frequency_mhz: f32,
    cpu_min_frequency_mhz: f32,
    bogo_mips: f32,
    flags: Vec<String>,
    virtualization: String,
    l1_cache_bytes: u64,
    l2_cache_bytes: u64,
    l3_cache_bytes: u64,
    numa_nodes: u32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuLoadStructure {
    user_time: f32,
    system_time: f32,
    nice_time: f32,
    wait_time: f32,
    idle_time: f32,
    hardware_interrupts: f32,
    software_interrupts: f32,
    stolen_time: f32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    uptime: u64,
    load_avg_1m: f32,
    load_avg_5m: f32,
    load_avg_15m: f32,
    load_profile_avg: CpuLoadStructure,
    load_profile_per_cpu: Vec<CpuLoadStructure>,
    tasks_total: u32,
    tasks_running: u32,
    tasks_sleeping: u32,
    tasks_stopped: u32,
    tasks_zombie: u32
}

impl ByteOrder {
    pub fn current() -> ByteOrder {
        if cfg!(target_endian = "little") {
            ByteOrder::LittleEndian
        } else if cfg!(target_endian = "big") {
            ByteOrder::BigEndian
        } else {
            ByteOrder::Unknown
        }
    }
}

impl Cpu {
    pub fn get_info() -> io::Result<Cpu> {
        // Execute lscpu command
        let output = Command::new("lscpu")
            .arg("--bytes")  // Show sizes in bytes
            .output()?;
        
        if !output.status.success() {
            return Err(Error::new(ErrorKind::Other, "Failed to execute lscpu"));
        }

        let lscpu_output = String::from_utf8_lossy(&output.stdout);
        let info = parse_lscpu(&lscpu_output);

        // Parse key values
        let sockets = info.get("Socket(s)")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        
        let cores_per_socket = info.get("Core(s) per socket")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let threads_per_core = info.get("Thread(s) per core")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let numa_nodes = info.get("NUMA node(s)")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        // Parse cache sizes
        let (l1, l2, l3) = parse_cache_sizes(&info);
        let flags = info.get("Flags")
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default();

        Ok(Cpu {
            architecture: match_cpu_arch(info.get("Architecture")
                .unwrap_or(&String::new())),

            vendor_id: info.get("Vendor ID").cloned().unwrap_or_default(),
            model: info.get("Model name").cloned().unwrap_or_default(),
            cpu_family: info.get("CPU family").and_then(|s| s.parse().ok())
                .unwrap_or(0),

            op_modes: info.get("CPU op-mode(s)")
                .map(|s| s.split(',').map(|m| m.trim().into()).collect())
                .unwrap_or_default(),

            address_size: info.get("Address sizes")
                .and_then(|s| s.split(',').next())
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),

            byte_order: match info.get("Byte Order").map(|s| s.as_str()) {
                Some("Little Endian") => ByteOrder::LittleEndian,
                Some("Big Endian") => ByteOrder::BigEndian,
                _ => ByteOrder::Unknown,
            },

            cores: sockets * cores_per_socket,
            threads_per_core,
            cores_per_socket,
            sockets,
            stepping: info.get("Stepping").and_then(|s| s.parse().ok())
                .unwrap_or(0),

            l1_cache_bytes: l1,
            l2_cache_bytes: l2,
            l3_cache_bytes: l3,
            flags,
            virtualization: info.get("Virtualization").cloned()
                .unwrap_or_default(),

            bogo_mips: info.get("BogoMIPS").and_then(|s| s.parse().ok())
                .unwrap_or(0.0),

            numa_nodes,
            ..Default::default()
        })
    }
}

fn parse_lscpu(output: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in output.lines() {
        if let Some((key, value)) = line.split_once(':') {
            map.insert(
                key.trim().to_string(),
                value.trim().replace(",", "").to_string(),
            );
        }
    }
    map
}

fn parse_cache_sizes(info: &HashMap<String, String>) -> (u64, u64, u64) {
    let parse = |key: &str| -> u64 {
        info.get(key)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    };

    let l1d = parse("L1d cache");
    let l1i = parse("L1i cache");
    let l2 = parse("L2 cache");
    let l3 = parse("L3 cache");

    (l1d + l1i, l2, l3)
}


impl Architecture {
    pub fn current() -> Architecture {
        match read_to_string("/proc/sys/kernel/arch") {
            Ok(arch) => match_cpu_arch(&arch),
            Err(_) => Architecture::Unknown
        }
    }
}

pub fn match_cpu_arch(arch: &str) -> Architecture {
    match arch.to_lowercase().as_str() {
        "x86_64" | "amd64" | "x64" => Architecture::AMD64,
        "x86" | "intel386" | "i386" => Architecture::I386,
        "intel686" | "i686" => Architecture::I686,
        "aarch64" | "arm64" => Architecture::ARM64,
        "aarch32" | "arm32" => Architecture::ARM32,
        "risc-v" | "riscv" | "risc-v64" | "riscv64" => Architecture::RISCV,
        "powerpc64" | "ppc64" | "ppc64le" => Architecture::PPC64,
        "s390x" | "s360x" => Architecture::S390X,
        "sparc64" => Architecture::SPARC64,
        _ => Architecture::Unknown
    }
}