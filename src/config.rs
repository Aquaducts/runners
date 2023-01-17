use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Server {
    pub host: String,
    pub port: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RunnerAllowedContainer {
    pub image: String,
    pub release: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Runner {
    pub containers: Vec<RunnerAllowedContainer>,
    pub backend: String,
    pub container_whitelist: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub server: Server,
}

pub const CONFIG: Lazy<Config> = Lazy::new(|| {
    toml::from_str::<Config>(&std::fs::read_to_string("./Config.toml").unwrap()).unwrap()
});
