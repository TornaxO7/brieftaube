use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "config.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub address: String,
    pub password: String,
    pub host: String,
}

impl Config {
    // TODO: Error handling
    pub fn load() -> Result<Self, ()> {
        let xdg = crate::get_xdg();

        let path = xdg.place_config_file(FILE_NAME).unwrap();

        let config_content = std::fs::read_to_string(path).unwrap();

        Ok(toml::from_str(&config_content).unwrap())
    }
}
