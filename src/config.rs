use crate::{fish_config::FishConfig, scene_config::SceneConfig};
use macroquad::{
    file::load_string,
    texture::{load_texture, Texture2D},
};
use nanoserde::DeJson;
use std::collections::HashMap;

#[derive(DeJson, Debug)]
pub struct Config {
    #[nserde(default = "inputdata.json")]
    pub input_data_path: Option<String>,
    pub display_time: u32,
    pub backgrounds: Vec<String>,
    pub scenes: Option<Vec<SceneConfig>>,
    pub fishes: HashMap<String, FishConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input_data_path: Some("inputdata.json".to_string()),
            display_time: 0,
            backgrounds: vec![],
            scenes: None,
            fishes: HashMap::new(),
        }
    }
}

impl Config {
    pub async fn load() -> Self {
        let json = load_string("config.json")
            .await
            .unwrap_or_else(|_| "{}".to_string());
        DeJson::deserialize_json(&json).expect("Failed parsing config")
    }

    pub async fn background_textures(&self) -> Vec<Texture2D> {
        let background_futures = self
            .backgrounds
            .iter()
            .map(|background| load_texture(background));
        futures::future::join_all(background_futures)
            .await
            .into_iter()
            .flatten()
            .collect()
    }
}
