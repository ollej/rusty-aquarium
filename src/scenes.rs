use crate::{legend::Legend, scene_config::SceneConfig};
use macroquad::{
    color::colors::WHITE,
    math::Vec2,
    texture::{draw_texture_ex, DrawTextureParams, Texture2D},
};

pub struct Scenes {
    current_scene: usize,
    scenes: Vec<SceneConfig>,
    backgrounds: Vec<Texture2D>,
    time: f32,
    switching: bool,
}

impl Scenes {
    pub fn new(scenes: Vec<SceneConfig>, backgrounds: Vec<Texture2D>) -> Self {
        Self {
            current_scene: 0,
            scenes,
            backgrounds,
            time: 0.,
            switching: true,
        }
    }

    pub fn empty() -> Self {
        Scenes {
            current_scene: 0,
            scenes: vec![SceneConfig::default()],
            backgrounds: vec![],
            time: 0.,
            switching: false,
        }
    }

    pub fn is_switching(&self) -> bool {
        self.switching && self.display_time() > 0
    }

    pub fn tick(&mut self, delta: f32) {
        self.time += delta;
    }

    pub fn needs_reloading(&self) -> bool {
        self.is_switching() && self.time > self.display_time() as f32
    }

    pub fn draw(&self, rect: Vec2) {
        draw_texture_ex(
            self.backgrounds[self.scene_background()],
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(rect),
                ..Default::default()
            },
        );
    }

    fn display_time(&self) -> u32 {
        self.scenes[self.current_scene].display_time
    }

    fn scene_background(&self) -> usize {
        self.scenes[self.current_scene].background.unwrap_or(0)
    }

    pub fn input_data_path(&self) -> Option<String> {
        self.scenes[self.current_scene].input_data_path.clone()
    }

    pub fn legend(&self) -> Option<Legend> {
        self.scenes[self.current_scene].legend.clone()
    }

    pub fn next(&mut self) {
        self.time = 0.;
        self.current_scene += 1;
        if self.current_scene == self.scenes.len() {
            self.current_scene = 0;
        }
    }

    pub fn toggle_switching(&mut self) -> bool {
        self.time = 0.;
        self.switching = !self.switching;
        self.switching
    }
}
