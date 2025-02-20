use crate::{fish_legend::FishLegend, legend::Legend, resources::Resources};
use macroquad::{
    color::{Color, colors::WHITE},
    experimental::collections::storage,
    math::vec2,
    shapes::draw_rectangle,
    text::{draw_text, measure_text},
    texture::{DrawTextureParams, Texture2D, draw_texture_ex},
    window::{screen_height, screen_width},
};
use std::collections::HashMap;

pub struct ShowLegend {
    pub showing: bool,
    default_legend: Option<Legend>,
    current_legend: Option<Legend>,
}

impl ShowLegend {
    const BACKGROUND_COLOR: Color = Color::new(0.1, 0.1, 0.1, 0.5);
    const FONT_COLOR: Color = WHITE;
    const MARGIN: f32 = 50.;
    const FONT_SIZE: f32 = 40.;
    const LINE_OFFSET: f32 = 10.;
    const FISH_SIZE: f32 = 75.;

    pub fn new(legend: Option<Legend>) -> Self {
        Self {
            showing: false,
            default_legend: legend,
            current_legend: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            showing: false,
            default_legend: None,
            current_legend: None,
        }
    }

    pub fn draw(&self) {
        if !self.showing {
            return;
        }
        if let Some(legend) = &self.current_legend {
            draw_rectangle(
                Self::MARGIN,
                Self::MARGIN,
                screen_width() - Self::MARGIN * 2.,
                screen_height() - Self::MARGIN * 2.,
                Self::BACKGROUND_COLOR,
            );

            let mut offset_y = Self::MARGIN * 2.;
            for line in legend.description.split('\n') {
                offset_y = self.draw_line(Self::MARGIN * 2., offset_y, line);
            }

            self.draw_fishes(offset_y, &legend.fish_legends);
        }
    }

    pub fn hide(&mut self) {
        self.showing = false;
        self.current_legend = None;
    }

    pub fn toggle_show(&mut self, scene_legend: Option<Legend>) {
        self.showing = !self.showing;
        if self.showing {
            self.current_legend = self.default_legend.clone().or(scene_legend);
        }
    }

    fn draw_line(&self, offset_x: f32, offset_y: f32, text: &str) -> f32 {
        draw_text(text, offset_x, offset_y, Self::FONT_SIZE, Self::FONT_COLOR);
        offset_y + Self::FONT_SIZE + Self::LINE_OFFSET
    }

    fn draw_fishes(&self, start_y: f32, fish_legends: &[FishLegend]) {
        let resources = storage::get::<Resources>();
        let fish_textures = &resources.fish_textures;
        let fish_configs = &resources.config.fishes;
        let max_fish_height = self.find_max_height(fish_textures);
        let mut offset_y = start_y;
        for fish_legend in fish_legends.iter() {
            if let Some(fish_config) = fish_configs.get(&fish_legend.fish) {
                if let Some(texture) = fish_textures.get(&fish_config.texture) {
                    offset_y =
                        self.draw_fish_legend(offset_y, max_fish_height, fish_legend, texture);
                }
            }
        }
    }

    fn draw_fish_legend(
        &self,
        offset_y: f32,
        max_fish_height: f32,
        fish_legend: &FishLegend,
        texture: &Texture2D,
    ) -> f32 {
        let fish_height = Self::FISH_SIZE / (texture.width() / texture.height());
        draw_texture_ex(
            texture,
            Self::MARGIN * 2.,
            offset_y + (max_fish_height - fish_height) / 2.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(Self::FISH_SIZE, fish_height)),
                ..Default::default()
            },
        );

        let text_dim = measure_text(&fish_legend.description, None, Self::FONT_SIZE as u16, 1.0);
        let text_middle = text_dim.height / 2.0;
        let below_baseline = text_dim.height - text_dim.offset_y;
        self.draw_line(
            Self::MARGIN * 2. + Self::FISH_SIZE * 1.5,
            offset_y - below_baseline + text_middle + max_fish_height / 2.,
            &fish_legend.description,
        );

        offset_y + max_fish_height + Self::LINE_OFFSET
    }

    fn find_max_height(&self, fish_textures: &HashMap<String, Texture2D>) -> f32 {
        fish_textures
            .values()
            .map(|texture| Self::FISH_SIZE / (texture.width() / texture.height()))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap()
    }
}
