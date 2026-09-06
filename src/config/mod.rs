use serde::{Deserialize, Serialize};

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

    pub fn html_renderer(&self) -> String {
        self.html_renderer.clone().unwrap_or("xdg-open".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub username: String,
    pub password: String,
    pub host: String,

    #[serde(default)]
    pub cache: Cache,

    #[serde(default)]
    pub backend: Backend,
}
