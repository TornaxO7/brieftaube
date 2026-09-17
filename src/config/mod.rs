mod types;

use serde::{Deserialize, Serialize};

pub use types::*;

pub const FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Cache {
    #[default]
    Internal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Backend {
    #[default]
    Jmap,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub users: Vec<UserConfig>,
    html_renderer: Option<String>,
    editor: Option<String>,
}

impl Config {
    pub fn editor(&self) -> Option<String> {
        self.editor.clone().or_else(|| std::env::var("EDITOR").ok())
    }

    pub fn html_renderer(&self) -> Option<String> {
        self.html_renderer.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub username: Username,
    pub password: String,
    pub server_url: String,

    #[serde(default)]
    pub cache: Cache,

    #[serde(default)]
    pub backend: Backend,
}
