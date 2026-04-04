use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub initial_address: String,
    pub initial_supply: u128,
    pub chain_id: u64,
}

impl GenesisConfig {
    pub fn load(path: &str) -> Result<GenesisConfig, String> {
        let contents = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&contents).map_err(|e| e.to_string())
    }
}