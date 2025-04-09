#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mount {
    filesystem: String,
    mount_point: String,
    mount_type: String,
    mount_options: Vec<String>,
    size: u64,
    free: u64,
    used: u64
}