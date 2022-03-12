use once_cell::sync::Lazy;
use serde_derive::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    pub token: String,
    pub key: String,
    pub shards: u64,
    pub data: String,
    pub debug: bool,
    pub cache: Cache,
    pub infos: Infos,
    pub js: Js,
}

#[derive(Deserialize)]
pub struct Cache {
    pub time: u64,
    pub size: usize,
}

#[derive(Deserialize)]
pub struct Infos {
    pub name: String,
    pub prefix: String,
    pub activity: String,
    pub help: String,
    pub image: String,   
}

#[derive(Deserialize)]
pub struct Js {
    pub enabled: bool,
    pub timeout: u64,
    pub memory: usize,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| init());

pub fn init() -> Config {
    let file = fs::read_to_string("./config.yaml").expect("Conf file read error");
    serde_yaml::from_str(&file).unwrap()
}
