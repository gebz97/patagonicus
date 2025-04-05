// ======================
// BYTE UNITS (BYTES)
// ======================
pub const B: u64   = 1;
pub const KB: u64  = 1000 * B;
pub const MB: u64  = 1000 * KB;
pub const GB: u64  = 1000 * MB;
pub const TB: u64  = 1000 * GB;
pub const PB: u64  = 1000 * TB;
pub const EB: u64  = 1000 * PB;

// ======================
// BINARY BYTE UNITS (BYTES)
// ======================
pub const KIB: u64  = 1024 * B;
pub const MIB: u64  = 1024 * KIB;
pub const GIB: u64  = 1024 * MIB;
pub const TIB: u64  = 1024 * GIB;
pub const PIB: u64  = 1024 * TIB;
pub const EIB: u64  = 1024 * PIB;

// ======================
// BITRATE UNITS (BITS PER SECOND)
// ======================
pub const BPS: u64   = 1;
pub const KBPS: u64  = 1000 * BPS;
pub const MBPS: u64  = 1000 * KBPS;
pub const GBPS: u64  = 1000 * MBPS;
pub const TBPS: u64  = 1000 * GBPS;
pub const PBPS: u64  = 1000 * TBPS;
pub const EBPS: u64  = 1000 * PBPS;

// ======================
// BINARY BITRATE UNITS (BINARY BITS PER SECOND)
// ======================
pub const KIBPS: u64  = 1024 * BPS;
pub const MIBPS: u64  = 1024 * KIBPS;
pub const GIBPS: u64  = 1024 * MIBPS;
pub const TIBPS: u64  = 1024 * GIBPS;
pub const PIBPS: u64  = 1024 * TIBPS;
pub const EIBPS: u64  = 1024 * PIBPS;

/// Convert a value from base unit to target unit (returns float)
pub fn to_unit(value: u64, unit: u64) -> f64 {
    value as f64 / unit as f64
}

/// Convert a value from target unit to base unit (returns integer)
pub fn from_unit(value: f64, unit: u64) -> u64 {
    (value * unit as f64) as u64
}

/// Convert bits per second (BPS) to binary bits per second (KIBPS, MIBPS, etc.)
pub fn bps_to_kibps(bps: u64) -> u64 {
    bps / KIBPS
}

pub fn bps_to_mibps(bps: u64) -> u64 {
    bps / MIBPS
}

pub fn bps_to_gibps(bps: u64) -> u64 {
    bps / GIBPS
}

pub fn bps_to_tibps(bps: u64) -> u64 {
    bps / TIBPS
}

pub fn bps_to_pibps(bps: u64) -> u64 {
    bps / PIBPS
}

pub fn bps_to_eibps(bps: u64) -> u64 {
    bps / EIBPS
}

/// Convert binary bits per second (KIBPS, MIBPS, etc.) to bits per second (BPS)
pub fn kibps_to_bps(kibps: u64) -> u64 {
    kibps * KIBPS
}

pub fn mibps_to_bps(mibps: u64) -> u64 {
    mibps * MIBPS
}

pub fn gibps_to_bps(gibps: u64) -> u64 {
    gibps * GIBPS
}

pub fn tibps_to_bps(tibps: u64) -> u64 {
    tibps * TIBPS
}

pub fn pibps_to_bps(pibps: u64) -> u64 {
    pibps * PIBPS
}

pub fn eibps_to_bps(eibps: u64) -> u64 {
    eibps * EIBPS
}

pub fn human_readable_iec(bytes: u64) -> String {
    match bytes {
        b if b >= EIB => format!("{:.2} EiB", to_unit(b, EIB)),
        b if b >= PIB => format!("{:.2} PiB", to_unit(b, PIB)),
        b if b >= TIB => format!("{:.2} TiB", to_unit(b, TIB)),
        b if b >= GIB => format!("{:.2} GiB", to_unit(b, GIB)),
        b if b >= MIB => format!("{:.2} MiB", to_unit(b, MIB)),
        b if b >= KIB => format!("{:.2} KiB", to_unit(b, KIB)),
        _ => format!("{} B", bytes),
    }
}

pub fn human_readable_si(bytes: u64) -> String {
    match bytes {
        b if b >= EB => format!("{:.2} EB", to_unit(b, EB)),
        b if b >= PB => format!("{:.2} PB", to_unit(b, PB)),
        b if b >= TB => format!("{:.2} TB", to_unit(b, TB)),
        b if b >= GB => format!("{:.2} GB", to_unit(b, GB)),
        b if b >= MB => format!("{:.2} MB", to_unit(b, MB)),
        b if b >= KB => format!("{:.2} kB", to_unit(b, KB)),
        _ => format!("{} B", bytes),
    }
}

/// Convert to a human-readable bitrate in bits per second
pub fn human_readable_bitrate(bits_per_second: u64) -> String {
    match bits_per_second {
        b if b >= EBPS => format!("{:.2} Ebps", b as f64 / EBPS as f64),
        b if b >= PBPS => format!("{:.2} Pbps", b as f64 / PBPS as f64),
        b if b >= TBPS => format!("{:.2} Tbps", b as f64 / TBPS as f64),
        b if b >= GBPS => format!("{:.2} Gbps", b as f64 / GBPS as f64),
        b if b >= MBPS => format!("{:.2} Mbps", b as f64 / MBPS as f64),
        b if b >= KBPS => format!("{:.2} Kbps", b as f64 / KBPS as f64),
        _ => format!("{} bps", bits_per_second),
    }
}

/// Convert to a human-readable binary bitrate in bits per second
pub fn human_readable_binary_bitrate(bits_per_second: u64) -> String {
    match bits_per_second {
        b if b >= EIBPS => format!("{:.2} Eibps", b as f64 / EIBPS as f64),
        b if b >= PIBPS => format!("{:.2} Pibps", b as f64 / PIBPS as f64),
        b if b >= TIBPS => format!("{:.2} Tibps", b as f64 / TIBPS as f64),
        b if b >= GIBPS => format!("{:.2} Gibps", b as f64 / GIBPS as f64),
        b if b >= MIBPS => format!("{:.2} Mibps", b as f64 / MIBPS as f64),
        b if b >= KIBPS => format!("{:.2} Kibps", b as f64 / KIBPS as f64),
        _ => format!("{} bps", bits_per_second),
    }
}
