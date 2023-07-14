use crate::{
    config::Config, fish::Fish, fish_config::FishConfig, fish_data::FishData,
    input_data::InputData, resources::Resources, scene_config::SceneConfig, scenes::Scenes,
    show_legend::ShowLegend,
};
use macroquad::{
    experimental::{
        collections::storage,
        coroutines::{start_coroutine, Coroutine},
    },
    math::{Rect, Vec2},
    rand::ChooseRandom,
    texture::Texture2D,
};
use std::collections::HashMap;

pub struct FishTank {
    fishes: Vec<Fish>,
    fish_configs: HashMap<String, FishConfig>,
    fish_keys: Vec<String>,
    school: Vec<FishData>,
    bubble_texture: Option<Texture2D>,
    fish_textures: HashMap<String, Texture2D>,
    scenes: Scenes,
    reloader: Option<Coroutine>,
    pub loaded: bool,
    show_legend: ShowLegend,
}

impl Default for FishTank {
    fn default() -> Self {
        Self::new()
    }
}

impl FishTank {
    pub fn new() -> Self {
        Self {
            fishes: vec![],
            fish_keys: vec![],
            fish_configs: HashMap::new(),
            school: vec![],
            bubble_texture: None,
            fish_textures: HashMap::new(),
            scenes: Scenes::empty(),
            reloader: None,
            loaded: false,
            show_legend: ShowLegend::empty(),
        }
    }

    pub fn add_resources(&mut self) {
        let resources = storage::get::<Resources>();
        storage::store(resources.input_data.clone());
        self.bubble_texture = Some(resources.bubble_texture.clone());
        self.fish_keys = Vec::from_iter(resources.config.fishes.keys().cloned());
        self.fish_configs = resources.config.fishes.clone();
        self.school = (*resources.input_data.school).to_vec();
        self.fish_textures = resources.fish_textures.clone();
        let scenes = resources.config.scenes.clone().unwrap_or_else(|| {
            vec![SceneConfig::new(
                resources.config.input_data_path.clone(),
                resources.config.display_time,
            )]
        });
        self.scenes = Scenes::new(scenes, (*resources.backgrounds).to_vec());
        self.show_legend = ShowLegend::new(resources.input_data.legend.clone());
        self.populate();
        self.loaded = true;
    }

    pub fn reload_data(&mut self) {
        if let Some(path) = self.scenes.input_data_path() {
            self.reloader = Some(start_coroutine(async move {
                let data = InputData::load(path).await;
                storage::store(data);
            }));
        }
    }

    pub fn update_config(&mut self, config: Config) {
        self.fish_configs = config.fishes;
        self.repopulate();
    }

    pub fn next_scene(&mut self) {
        if self.show_legend.showing {
            self.show_legend.hide();
        }
        self.scenes.next();
        self.reload_data();
    }

    pub fn toggle_switching_scenes(&mut self) -> bool {
        self.scenes.toggle_switching()
    }

    pub fn toggle_legend(&mut self) {
        self.show_legend.toggle_show(self.scenes.legend());
    }

    pub fn tick(&mut self, delta: f32) {
        self.tick_data_reloading(delta);
        let collision_boxes = self
            .fishes
            .iter()
            .map(|fish| fish.collision_box())
            .collect::<Vec<Rect>>();
        for fish in self.fishes.iter_mut() {
            fish.tick(delta, &collision_boxes);
        }
    }

    pub fn draw(&mut self, rect: Vec2) {
        self.scenes.draw(rect);
        for fish in self.fishes.iter_mut() {
            fish.draw();
        }
    }

    pub fn draw_legend(&self) {
        self.show_legend.draw();
    }

    pub fn repopulate(&mut self) {
        self.reset();
        self.populate();
    }

    pub fn add_fish(&mut self) {
        let fish = self.random_fish();
        self.fishes.push(fish);
    }

    pub fn remove_fish(&mut self) {
        if !self.fishes.is_empty() {
            self.fishes.pop();
        }
    }

    fn tick_data_reloading(&mut self, delta: f32) {
        if let Some(reloader) = self.reloader {
            if reloader.is_done() {
                self.update_data();
                self.reloader = None;
            } else {
                return;
            }
        }
        if !self.scenes.is_switching() {
            return;
        }
        self.scenes.tick(delta);
        if self.scenes.needs_reloading() {
            self.next_scene();
        }
    }

    fn update_data(&mut self) {
        let input_data = storage::get_mut::<InputData>();
        self.school = (*input_data.school).to_vec();
        self.show_legend = ShowLegend::new(input_data.legend.clone());
        self.repopulate();
    }

    fn populate(&mut self) {
        for fish_data in self.school.iter() {
            if let Ok(fish) = self.create_fish(fish_data) {
                self.fishes.push(fish);
            }
        }
    }

    fn reset(&mut self) {
        self.fishes.clear();
    }

    fn random_fish_config(&self) -> &FishConfig {
        let fish_key = self.fish_keys.choose().unwrap();
        return self.fish_configs.get(fish_key).unwrap();
    }

    fn random_fish(&self) -> Fish {
        let fish_config = self.random_fish_config();
        Fish::new(
            fish_config.randomized_size(),
            fish_config.randomized_speed(),
            fish_config.collision_aversion,
            fish_config.area,
            fish_config.movement,
            self.fish_textures
                .get(&fish_config.texture)
                .unwrap()
                .clone(),
            self.bubble_texture.clone().unwrap(),
            fish_config.randomized_bubble_amount(),
        )
    }

    fn create_fish(&self, fish_data: &FishData) -> Result<Fish, &'static str> {
        let fish_config = self
            .fish_configs
            .get(&fish_data.fish)
            .ok_or("FishConfig missing")?;
        Ok(Fish::new(
            fish_config.size * fish_data.size,
            fish_config.speed * fish_data.speed,
            fish_config.collision_aversion,
            fish_config.area,
            fish_config.movement,
            self.fish_textures
                .get(&fish_config.texture)
                .unwrap()
                .clone(),
            self.bubble_texture.clone().unwrap(),
            fish_config.bubbles * fish_data.bubbles as u32,
        ))
    }
}
