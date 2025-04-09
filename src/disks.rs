use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DiskLabel {
    GPT,
    MBR,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Disk {
    name: String,
    uuid: String,
    model: String,
    disklabel_type: String,
    size: u64,
    sector_size: u64,
    n_sectors: u64,
    io_size: u32,
    partitions: Vec<Partition>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Partition {
    name: String,
    start: u64,
    end: u64,
    sectors: u64,
    size: u64,
    uuid: String,
    part_type: String,
}

impl DiskLabel {
    pub fn to_string(&self) -> String {
        match self {
            Self::GPT => String::from("GPT"),
            Self::MBR => String::from("MBR"),
            Self::Unknown => String::from("Unknown")
        }
    }
}

impl Partition {
    pub fn new(device: &str, part: &str) -> io::Result<Self> {
        let partition_path = Path::new("/sys/block").join(device).join(part);

        if !partition_path.is_dir() {
            return Err(
                io::Error::new(io::ErrorKind::NotFound,"Partition not found"
            ));
        }

        let uuid = match get_uuid_from_dir("/dev/disk/by-uuid", part) {
            Ok(Some(uuid)) => uuid,
            Ok(None) => {
                format!("UUID not found for partition: {}", part)
            }
            Err(e) => format!("Error while parsing the UUID for {}: {}", part, e)
        };        
        
        let part_type = get_partition_type(&part)
            .expect(&format!("Unable to get partition type {}", part))
            .unwrap_or("Unknown partition type.".to_string());

        let (sectors, start, end) = get_partition_sectors(device, part)
            .expect(&format!("Unable to get sector info: {}", part));

        let size = sectors * get_sector_size(&device)
            .expect(&format!("Unable to get device size {}", part));

        Ok(Partition {
            name: part.to_string(),
            start,
            end,
            sectors,
            size,
            uuid,
            part_type,
        })
    }
}

impl Disk {
    pub fn new(device: &str) -> io::Result<Self> {
        let device_path = Path::new("/sys/block").join(device);

        if !device_path.is_dir() {
            return Err(
                io::Error::new(io::ErrorKind::NotFound, "Device not found"
            ));
        }

        let uuid = get_device_uuid(device)
            .expect(&format!("Unable to get device UUID {}", &device));

        let model = get_device_model(device)
            .expect(&format!("Unable to get device model {}", &device));

        let disklabel_type = detect_disklabel(device)
            .expect(&format!("Unable to get disk label type {}", &device))
            .to_string();

        let size = read_capacity(device)
            .expect(&format!("Unable to get capacity {}", &device));

        let sector_size = get_sector_size(device)
            .expect(&format!("Unable to get sector size {}", &device));

        let n_sectors = size / sector_size as u64;
        let io_size = get_io_size(device)
            .expect(&format!("Unable to get io size {}", &device));

        let partitions = get_partitions(device)
            .into_iter()
            .filter(|part| !part.contains("loop"))
            .map(|part| Partition::new(device, &part))
            .collect::<io::Result<Vec<Partition>>>()
            .expect(&format!("Unable to get partitions {}", &device));

        Ok(Disk {
            name: device.to_string(),
            uuid,
            model,
            disklabel_type,
            size,
            sector_size,
            n_sectors,
            io_size,
            partitions,
        })
    }
}


pub fn get_block_devices() -> Vec<String> {
    let mut block_devices = Vec::new();
    
    // Read the contents of /sys/block
    if let Ok(entries) = fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let device_name = entry.file_name();

            if !device_name.to_string_lossy().starts_with("loop") {
                let device_path = format!("{}", device_name.to_string_lossy());
                block_devices.push(device_path);
            }
        }
    }
    
    block_devices
}

pub fn get_partitions(device_name: &str) -> Vec<String> {
    let device_path = Path::new("/sys/block").join(device_name);

    if !device_path.is_dir() {
        return Vec::new();
    }

    let mut partitions = Vec::new();

    if let Ok(entries) = fs::read_dir(device_path) {
        for entry in entries.flatten() {
            let partition_name = entry.file_name();

            if partition_name != device_name 
                && partition_name.to_string_lossy().starts_with(device_name) {
                let partition_path = format!("{}", 
                    partition_name.to_string_lossy());
                partitions.push(partition_path);
            }
        }
    }

    partitions
}

pub fn detect_disklabel(device: &str) -> io::Result<DiskLabel> {
    let path = format!("/dev/{}", device);
    let mut file = File::open(&path)
        .expect(&format!("Unable to open file: {}", path));

    let mut mbr = [0u8; 512];
    file.read_exact(&mut mbr)
        .expect(&format!("Unable to read Disk data from: {}", path));

    if mbr[510] != 0x55 || mbr[511] != 0xAA {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid MBR signature"
        ));
    }

    let part_type = mbr[450];
    if part_type == 0xEE {
        file.seek(SeekFrom::Start(512))
            .expect(&format!(
                "Unable to do direct seek on mbr for: {}",
                &device
            ));

        let mut gpt = [0u8; 8];

        file.read_exact(&mut gpt)
            .expect(&format!("Unable to read exact bytes from: {}", &device));
        if &gpt == b"EFI PART" {
            return Ok(DiskLabel::GPT);
        }
    }
    Ok(DiskLabel::MBR)
}

pub fn get_sector_size(device: &str) -> io::Result<u64> {
    let path = format!("/sys/block/{}/queue/logical_block_size", device);
    let size_str = fs::read_to_string(&path)
        .expect(&format!("Unable to read file content: {}", &path));

    size_str.trim().parse().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData, format!("Invalid sector size: {}",
            e
        ))
    })
}

pub fn read_capacity(device: &str) -> io::Result<u64> {
    let sector_size = get_sector_size(device).unwrap_or(512);
    let path = format!("/sys/class/block/{}/size", device);
    
    let size_str = fs::read_to_string(path)
        .expect(&format!("Unable to open path: {}", &device));

    let capacity_in_sectors = size_str.trim().parse::<u64>().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData, format!("Invalid capacity: {}", e)
        )
    }).expect(&format!("Unable to parse capacity for {}", &device));

    Ok(capacity_in_sectors * sector_size)
}

pub fn get_device_model(device: &str) -> io::Result<String> {
    let path = format!("/sys/block/{}/device/model", device);
    let model = fs::read_to_string(&path)
        .expect(&format!("Unable to read contents of: {}", &path));

    Ok(model.trim().to_string())
}

pub fn get_uuid_from_dir(
        path: &str, device: &str
    ) -> io::Result<Option<String>> {
    if Path::new(path).exists() {
        for entry_result in fs::read_dir(path)
            .expect(&format!("Unable to read UUID Path: {}", &path)) {

            let entry = entry_result
                .expect(
                    &format!("No entry found for UUID Link Lookup: {}", &path)
                );

            let target = fs::read_link(entry.path())
                .expect(&format!("Unable to read link: {}", &path));

            if target.to_string_lossy().contains(device) {
                if let Some(uuid) = entry.file_name().to_str() {
                    return Ok(Some(uuid.to_string()));
                }
            }
        }
    }
    Ok(None)
}


pub fn get_device_uuid(device: &str) -> io::Result<String> {
    if let Some(uuid) = get_uuid_from_dir("/dev/disk/by-id", device)
        .expect(&format!("Unable to get disk UUID: {}", &device)) {
        if let Some(id) = uuid.split('-').last() {
            return Ok(id.to_string());
        }
    }

    if let Some(uuid) = get_uuid_from_dir("/dev/disk/by-uuid", device)
        .expect(&format!("Unable to get device UUID {}", &device)) {
        return Ok(uuid);
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "UUID not found"))
}

pub fn get_io_size(device: &str) -> io::Result<u32> {
    let path = format!("/sys/block/{}/queue/optimal_io_size", device);
    let io_size_str = fs::read_to_string(&path)
        .expect(&format!("Unable to read contents of: {}", &path));

    io_size_str.trim().parse().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData, format!("Invalid IO size: {}", e)
        )
    })
}

pub fn get_partition_sectors(
        device: &str, partition: &str
    ) -> io::Result<(u64, u64, u64)> {
    let partition_path = Path::new("/sys/block").join(device).join(partition);

    if !partition_path.is_dir() {
        return Err(
            io::Error::new(io::ErrorKind::NotFound, "Partition not found")
        );
    }

    let start = fs::read_to_string(partition_path.join("start"))
        .expect(&format!("Unable to get partition sectors for: {}", &partition))
        .trim()
        .parse::<u64>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
            format!("Invalid start sector: {}", e)))
        .expect(&format!("Unable to fetch sector start: {}", &partition));
    
    let size = fs::read_to_string(partition_path.join("size"))
        .expect(&format!("Unable to get sector size for: {}", &partition))
        .trim()
        .parse::<u64>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
            format!("Invalid size: {}", e)))
        .expect(&format!("Unable to fetch sector size: {}", &partition));

    let end = start + size - 1;
    
    let sectors = size;

    Ok((sectors, start, end))
}

pub fn get_partition_type(device: &str) -> io::Result<Option<String>> {
    let partition_path = format!("/sys/class/block/{}/partition", device);
    
    if !Path::new(&partition_path).exists() {
        return Err(
            io::Error::new(io::ErrorKind::NotFound, "Partition path not found")
        );
    }
    
    let mut file = match fs::File::open(&partition_path) {
        Ok(f) => f,
        Err(e) => return Err(io::Error::new(io::ErrorKind::NotFound,
            format!("Failed to open partition file: {}", e))),
    };

    let mut buffer = String::new();
    
    match file.read_to_string(&mut buffer) {
        Ok(_) => (),
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, 
            format!("Failed to read partition file: {}", e))),
    }

    if buffer.is_empty() {
        Err( io::Error::new(
            io::ErrorKind::Other, "Partition type is empty or unreadable"
        ))
    } else {
        Ok(Some(buffer.trim().to_string()))
    }
}