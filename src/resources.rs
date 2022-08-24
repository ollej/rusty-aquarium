use crate::{config::Config, fish::Fish, input_data::InputData};
use macroquad::{
    color::colors::{BLACK, WHITE},
    experimental::{collections::storage, coroutines::start_coroutine},
    text::{draw_text, measure_text},
    texture::{load_texture, Texture2D},
    time::get_time,
    window::{clear_background, next_frame, screen_height, screen_width},
};
use std::collections::HashMap;

pub struct Resources {
    pub config: Config,
    pub input_data: InputData,
    pub backgrounds: Vec<Texture2D>,
    pub bubble_texture: Texture2D,
    pub fish_textures: HashMap<String, Texture2D>,
}

impl Resources {
    pub async fn new() -> Result<Resources, macroquad::prelude::FileError> {
        let config = Config::load().await;
        let input_data_path = config
            .input_data_path
            .to_owned()
            .expect("input_data_path missing");
        let input_data = InputData::load(input_data_path).await;
        let bubble_texture: Texture2D = load_texture(Fish::SPRITE_WATER).await?;
        let backgrounds = config.background_textures().await;
        let mut fish_textures = HashMap::new();
        for (_key, fish) in config.fishes.iter() {
            fish_textures.insert(fish.texture.clone(), load_texture(&fish.texture).await?);
        }

        Ok(Resources {
            config,
            input_data,
            backgrounds,
            bubble_texture,
            fish_textures,
        })
    }

    pub async fn load() {
        let resources_loading = start_coroutine(async move {
            let resources = Resources::new().await.unwrap();
            storage::store(resources);
        });

        let text_dim = measure_text("Filling up fish tank ...", None, 40, 1.0);
        while !resources_loading.is_done() {
            clear_background(BLACK);
            draw_text(
                &format!(
                    "Filling up fish tank {}",
                    ".".repeat(((get_time() * 2.0) as usize) % 4)
                ),
                screen_width() / 2.0 - text_dim.width / 2.,
                screen_height() / 2.0,
                40.,
                WHITE,
            );

            next_frame().await;
        }
    }
}
