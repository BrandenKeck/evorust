use std::fs;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct General {
    pub grid_w: usize,
    pub grid_h: usize,
    pub csize: f32,
    pub hist: usize,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: General,
}

pub fn get_config() -> Config {
    let text = include_str!("config.yaml");
    // let text = fs::read_to_string("config.yaml").unwrap();
    let config: Config = serde_saphyr::from_str(&text).unwrap();
    config
}