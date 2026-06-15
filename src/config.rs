use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub socket_path: String,
    pub max_concurrent_connections: usize,
    pub timeout_ms: u64,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = Path::new("config.json");
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_json::from_str(&contents)?;
    Ok(config)
}
