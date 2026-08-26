mod reader;

pub use reader::*;
use serde::{Deserialize, Serialize};

pub const FILE_NAME: &str = "config.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub address: String,
    pub password: String,
    pub host: String,
    html_renderer: Option<String>,
    editor: Option<String>,

    pub reader: Reader,
}

impl Config {
    pub fn editor(&self) -> Option<String> {
        self.editor.clone().or_else(|| std::env::var("EDITOR").ok())
    }

    pub fn html_renderer(&self) -> String {
        self.html_renderer.clone().unwrap_or("xdg-open".to_string())
    }
}
