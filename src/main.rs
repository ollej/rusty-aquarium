pub mod shaders;
use futures::future::join_all;
use macroquad::experimental::collections::storage;
use macroquad::experimental::coroutines::{start_coroutine, Coroutine};
use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;
use macroquad_particles::{AtlasConfig, BlendMode, Emitter, EmitterConfig};
use nanoserde::DeJson;
use quad_net::http_request::RequestBuilder;
use std::collections::HashMap;
use std::iter::FromIterator;

pub enum Collision {
    Left,
    Right,
    No,
}

#[derive(Copy, Clone)]
pub struct Motion {
    position: Vec2,
    speed: Vec2,
    max_speed: Vec2,
    acceleration: Vec2,
    rotation: f32,
    idle: bool,
}

impl Motion {
    const MAX_ROTATION: f32 = 0.3;
    const DIRECTION_CHANGE_CHANCE_X: f32 = 2.5;
    const DIRECTION_CHANGE_CHANCE_Y: f32 = 5.;

    fn move_position(&mut self, delta: f32, motion: Motion, bounding_box: Rect) -> Motion {
        //debug!("x: {} y: {} d: {}", self.position.x, self.position.y, delta);

        let position = self.clamp(
            if motion.idle {
                motion.position
            } else {
                motion.position + motion.speed * delta
            },
            bounding_box,
        );

        //debug!("rotation: {} new_pos: {} old_pos: {}", rotation, new_position, self.motion.position);
        Motion {
            position,
            speed: motion.speed,
            max_speed: motion.max_speed,
            acceleration: motion.acceleration,
            rotation: motion.rotation,
            idle: motion.idle,
        }
    }

    fn rotate(&mut self) {
        self.rotation = (self.speed.y / self.max_speed.y).abs() * Self::MAX_ROTATION;
        if self.speed.x * self.speed.y < 0. {
            self.rotation *= -1.;
        }
    }

    fn accelerate(&mut self) {
        if self.speed.x < self.max_speed.x && self.acceleration.x > 0. {
            self.speed.x += self.acceleration.x;
        }
        if self.speed.x > -self.max_speed.x && self.acceleration.x <= 0. {
            self.speed.x += self.acceleration.x;
        }
        if self.speed.y < self.max_speed.y && self.acceleration.y > 0. {
            self.speed.y += self.acceleration.y;
        }
        if self.speed.y > -self.max_speed.y && self.acceleration.y <= 0. {
            self.speed.y += self.acceleration.y;
        }
    }

    fn random_idling(&mut self) {
        if self.idle {
            self.idle ^= Self::random_percent() < Movement::CHANCE_IDLE_END;
        } else {
            self.idle ^= Self::random_percent() < Movement::CHANCE_IDLE_START;
        }
    }

    fn change_direction_by_bounding_box(&mut self, bounding_box: Rect) {
        if self.position.x <= bounding_box.x || self.position.x >= bounding_box.right() {
            self.speed.x *= -1.;
        }
        if self.position.y <= bounding_box.y || self.position.y >= bounding_box.bottom() {
            self.speed.y *= -1.;
        }
    }

    fn change_direction_vertically(&mut self, bounding_box: Rect) {
        if self.position.y <= bounding_box.y || self.position.y >= bounding_box.bottom() {
            self.speed.y *= -1.;
        }
    }

    fn change_acceleration_randomly(&mut self, multiplier: f32) {
        if Self::random_percent() < Self::DIRECTION_CHANGE_CHANCE_X * multiplier {
            self.acceleration.x *= -1.;
        }
        if Self::random_percent() < Self::DIRECTION_CHANGE_CHANCE_Y * multiplier {
            self.acceleration.y *= -1.;
        }
    }

    fn collision(&mut self, collision: Collision) {
        match collision {
            Collision::Left => {
                if self.speed.x < 0. {
                    self.speed.x *= -1.
                }
            }
            Collision::Right => {
                if self.speed.x > 0. {
                    self.speed.x *= -1.
                }
            }
            Collision::No => (),
        }
    }

    fn clamp(&self, position: Vec2, bounding_box: Rect) -> Vec2 {
        position
            .max(bounding_box.point())
            .min(vec2(bounding_box.right(), bounding_box.bottom()))
    }

    fn random_percent() -> f32 {
        rand::gen_range(0., 100.)
    }
}

#[derive(Debug, Copy, Clone, DeJson)]
pub enum Movement {
    SingleSpeed,
    Accelerating,
    AcceleratingEdgeIdling,
    Crab,
    Random,
}

impl Default for Movement {
    fn default() -> Self {
        Movement::Accelerating
    }
}

impl Movement {
    const CHANCE_IDLE_START: f32 = 0.05;
    const CHANCE_IDLE_END: f32 = 0.75;

    fn tick(&mut self, motion: Motion, bounding_box: Rect, collision: Collision) -> Motion {
        match self {
            Self::SingleSpeed => Self::tick_single_speed(motion, bounding_box, collision),
            Self::Accelerating => Self::tick_accelerating(motion, bounding_box, collision),
            Self::AcceleratingEdgeIdling => {
                Self::tick_accelerating_edge_idling(motion, bounding_box, collision)
            }
            Self::Crab => Self::tick_crab(motion, bounding_box, collision),
            Self::Random => Self::tick_random(motion, bounding_box, collision),
        }
    }

    #[allow(dead_code)]
    fn random() -> Self {
        *vec![
            Self::SingleSpeed,
            Self::Accelerating,
            Self::AcceleratingEdgeIdling,
            Self::Random,
        ]
        .choose()
        .unwrap()
    }

    fn tick_single_speed(mut motion: Motion, bounding_box: Rect, collision: Collision) -> Motion {
        motion.collision(collision);
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(1.);
        motion.rotate();
        motion
    }

    fn tick_accelerating(mut motion: Motion, bounding_box: Rect, collision: Collision) -> Motion {
        motion.collision(collision);
        motion.accelerate();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(1.);
        motion.rotate();
        motion
    }

    fn tick_accelerating_edge_idling(
        mut motion: Motion,
        bounding_box: Rect,
        collision: Collision,
    ) -> Motion {
        motion.collision(collision);
        motion.accelerate();
        motion.change_direction_vertically(bounding_box);
        motion.change_acceleration_randomly(1.);
        motion.rotate();
        motion
    }

    fn tick_crab(mut motion: Motion, bounding_box: Rect, collision: Collision) -> Motion {
        motion.collision(collision);
        motion.accelerate();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(5.);
        motion
    }

    fn tick_random(mut motion: Motion, bounding_box: Rect, collision: Collision) -> Motion {
        motion.collision(collision);
        motion.accelerate();
        motion.random_idling();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(1.);
        motion.rotate();
        motion
    }
}

pub struct Fish {
    motion: Motion,
    movement: Movement,
    size: Vec2,
    bubble_amount: u32,
    bounding_box_adjusted: Rect,
    collision_aversion: f32,
    already_collided: bool,
    texture: Texture2D,
    emitter: Emitter,
}

impl Fish {
    const COLLISION_SIZE_DIFFERENCE: f32 = 2.0;
    //const SPRITE_BUBBLE: &'static str = "bubble.png";
    const SPRITE_WATER: &'static str = "water.png";
    //const SPRITE_YELLOWSUBMARINE: &'static str = "yellowsubmarine.png";

    fn new(
        fish_size: f32,
        max_speed: Vec2,
        collision_aversion: f32,
        bounding_box: Rect,
        movement: Movement,
        texture: Texture2D,
        bubble_texture: Texture2D,
        bubble_amount: u32,
    ) -> Self {
        let fish_height = fish_size / (texture.width() / texture.height());
        let size = vec2(fish_size, fish_height);
        let bbox_adjusted = Self::adjust_bounding_box(bounding_box, size);
        Self {
            motion: Motion {
                position: Self::random_start_position(bbox_adjusted),
                speed: Self::random_start_direction(max_speed),
                max_speed,
                acceleration: Self::random_acceleration(),
                rotation: 0.,
                idle: false,
            },
            size,
            bubble_amount,
            bounding_box_adjusted: bbox_adjusted,
            collision_aversion,
            already_collided: false,
            movement,
            texture,
            emitter: Emitter::new(EmitterConfig {
                emitting: true,
                amount: bubble_amount,
                lifetime: 1.4,
                lifetime_randomness: 0.9,
                size: 1.5,
                size_randomness: 0.9,
                explosiveness: 0.9,
                initial_velocity: 5.0,
                initial_velocity_randomness: 0.8,
                initial_direction_spread: 0.5,
                gravity: vec2(0.0, -5.0),
                atlas: Some(AtlasConfig::new(4, 2, 0..8)),
                texture: Some(bubble_texture),
                material: Some(shaders::water_particle::material()),
                blend_mode: BlendMode::Additive,
                ..Default::default()
            }),
        }
    }

    fn adjust_bounding_box(bounding_box: Rect, size: Vec2) -> Rect {
        Rect {
            x: bounding_box.x,
            y: bounding_box.y,
            w: bounding_box.w - size.x,
            h: bounding_box.h - size.y,
        }
    }

    fn random_start_position(bounding_box: Rect) -> Vec2 {
        vec2(
            rand::gen_range(bounding_box.x, bounding_box.right()),
            rand::gen_range(bounding_box.y, bounding_box.bottom()),
        )
    }

    fn random_start_direction(max_speed: Vec2) -> Vec2 {
        max_speed
            * vec2(
                *vec![-1., 1.].choose().unwrap(),
                *vec![-1., 1.].choose().unwrap(),
            )
    }

    fn random_acceleration() -> Vec2 {
        vec2(rand::gen_range(0.1, 0.2), rand::gen_range(0.1, 0.2))
    }

    fn tick(&mut self, delta: f32, collision_boxes: &[Rect]) {
        let collision = self.collided(collision_boxes);
        let collision_box = self.collision_box();
        self.already_collided = collision_boxes
            .iter()
            .any(|cb| cb != &collision_box && cb.overlaps(&collision_box));
        let motion = self
            .movement
            .tick(self.motion, self.bounding_box_adjusted, collision);
        self.motion = self
            .motion
            .move_position(delta, motion, self.bounding_box_adjusted);
    }

    fn collided(&self, collision_boxes: &[Rect]) -> Collision {
        if self.already_collided {
            return Collision::No;
        }
        let collision_box = self.collision_box();
        for cbox in collision_boxes.iter() {
            if cbox.x != self.motion.position.x
                && cbox.y != self.motion.position.y
                && (cbox.w - collision_box.w).abs() < Fish::COLLISION_SIZE_DIFFERENCE
                && collision_box.overlaps(cbox)
                && rand::gen_range(0., 1.) > self.collision_aversion
            {
                return if cbox.x < self.motion.position.x {
                    Collision::Left
                } else {
                    Collision::Right
                };
            }
        }
        Collision::No
    }

    fn collision_box(&self) -> Rect {
        Rect {
            x: self.motion.position.x,
            y: self.motion.position.y,
            w: self.size.x,
            h: self.size.y,
        }
    }

    fn swims_right(&self) -> bool {
        self.motion.speed.x >= 0.
    }

    fn emit_position(&self) -> Vec2 {
        self.motion.position
            + if !self.swims_right() {
                vec2(self.size.x, 0.)
            } else {
                vec2(0., 0.)
            }
            + vec2(0., self.size.y / 2.)
    }

    fn emit(&mut self) {
        match self.movement {
            Movement::Crab => (),
            _ => {
                if self.bubble_amount > 0 {
                    self.emitter.draw(self.emit_position())
                }
            }
        }
    }

    fn draw(&mut self) {
        if !self.motion.idle {
            self.emit();
        }
        draw_texture_ex(
            self.texture,
            self.motion.position.x,
            self.motion.position.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.size),
                flip_x: self.swims_right(),
                rotation: self.motion.rotation,
                ..Default::default()
            },
        );
    }
}

struct FishTank {
    fishes: Vec<Fish>,
    fish_configs: HashMap<String, FishConfig>,
    fish_keys: Vec<String>,
    school: Vec<FishData>,
    bubble_texture: Option<Texture2D>,
    fish_textures: HashMap<String, Texture2D>,
    scenes: Scenes,
    reloader: Option<Coroutine>,
    loaded: bool,
    show_legend: ShowLegend,
}

impl FishTank {
    fn new() -> Self {
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

    fn add_resources(&mut self) {
        let resources = storage::get::<Resources>();
        storage::store(resources.input_data.clone());
        self.bubble_texture = Some(resources.bubble_texture);
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

    fn reload_data(&mut self) {
        if let Some(path) = self.scenes.input_data_path() {
            self.reloader = Some(start_coroutine(async move {
                let data = InputData::load(path).await;
                storage::store(data);
            }));
        }
    }

    fn update_data(&mut self) {
        let input_data = storage::get_mut::<InputData>();
        self.school = (*input_data.school).to_vec();
        self.show_legend = ShowLegend::new(input_data.legend.clone());
        self.repopulate();
    }

    fn update_config(&mut self, config: Config) {
        self.fish_configs = config.fishes;
        self.repopulate();
    }

    fn next_scene(&mut self) {
        if self.show_legend.showing {
            self.show_legend.hide();
        }
        self.scenes.next();
        self.reload_data();
    }

    fn toggle_switching_scenes(&mut self) -> bool {
        self.scenes.toggle_switching()
    }

    fn toggle_legend(&mut self) {
        self.show_legend.toggle_show(self.scenes.legend());
    }

    fn tick(&mut self, delta: f32) {
        self.tick_data_reloading(delta);
        let collision_boxes = self
            .fishes
            .iter()
            .map(|fish| fish.collision_box())
            .collect::<Vec<macroquad::math::Rect>>();
        for fish in self.fishes.iter_mut() {
            fish.tick(delta, &collision_boxes);
        }
    }

    fn draw(&mut self, rect: Vec2) {
        self.scenes.draw(rect);
        for fish in self.fishes.iter_mut() {
            fish.draw();
        }
    }

    fn draw_legend(&self) {
        self.show_legend.draw();
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

    fn repopulate(&mut self) {
        self.reset();
        self.populate();
    }

    fn add_fish(&mut self) {
        let fish = self.random_fish();
        self.fishes.push(fish);
    }

    fn remove_fish(&mut self) {
        if !self.fishes.is_empty() {
            self.fishes.pop();
        }
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
            *self.fish_textures.get(&fish_config.texture).unwrap(),
            self.bubble_texture.unwrap(),
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
            *self.fish_textures.get(&fish_config.texture).unwrap(),
            self.bubble_texture.unwrap(),
            fish_config.bubbles * fish_data.bubbles as u32,
        ))
    }
}

struct ShowText {
    text: &'static str,
    time: f32,
    x: f32,
    y: f32,
}

impl ShowText {
    fn new(text: &'static str) -> Self {
        Self {
            text,
            time: 2.,
            x: 20.,
            y: 40.,
        }
    }

    fn empty() -> Self {
        Self {
            text: "",
            time: 0.,
            x: 0.,
            y: 0.,
        }
    }

    fn draw(&mut self, delta: f32) {
        if self.time > 0. {
            self.time -= delta;
            draw_text(self.text, self.x, self.y, 40., WHITE);
        }
    }
}

struct ShowHelp {
    pub showing: bool,
}

impl ShowHelp {
    const BACKGROUND_COLOR: Color = Color::new(0.1, 0.1, 0.1, 0.5);
    const FONT_COLOR: Color = WHITE;
    const MARGIN: f32 = 50.;
    const FONT_SIZE: f32 = 40.;
    const LINE_OFFSET: f32 = 10.;
    const HELP_TEXT: &'static str = include_str!("helptext.txt");

    fn new() -> Self {
        Self { showing: false }
    }

    fn draw(&self) {
        if !self.showing {
            return;
        }
        draw_rectangle(
            Self::MARGIN,
            Self::MARGIN,
            screen_width() - Self::MARGIN * 2.,
            screen_height() - Self::MARGIN * 2.,
            Self::BACKGROUND_COLOR,
        );

        let mut offset_y = Self::MARGIN * 2.;
        for line in Self::HELP_TEXT.split('\n') {
            offset_y = self.draw_line(Self::MARGIN * 2., offset_y, line);
        }
    }

    fn draw_line(&self, offset_x: f32, offset_y: f32, text: &str) -> f32 {
        draw_text(text, offset_x, offset_y, Self::FONT_SIZE, Self::FONT_COLOR);
        offset_y + Self::FONT_SIZE + Self::LINE_OFFSET
    }

    fn toggle_show(&mut self) {
        self.showing = !self.showing
    }
}

struct ShowLegend {
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

    fn new(legend: Option<Legend>) -> Self {
        Self {
            showing: false,
            default_legend: legend,
            current_legend: None,
        }
    }

    fn empty() -> Self {
        Self {
            showing: false,
            default_legend: None,
            current_legend: None,
        }
    }

    fn draw(&self) {
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
            *texture,
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

    fn hide(&mut self) {
        self.showing = false;
        self.current_legend = None;
    }

    fn toggle_show(&mut self, scene_legend: Option<Legend>) {
        self.showing = !self.showing;
        if self.showing {
            self.current_legend = self.default_legend.clone().or(scene_legend);
        }
    }
}

struct Scenes {
    current_scene: usize,
    scenes: Vec<SceneConfig>,
    backgrounds: Vec<Texture2D>,
    time: f32,
    switching: bool,
}

impl Scenes {
    fn new(scenes: Vec<SceneConfig>, backgrounds: Vec<Texture2D>) -> Self {
        Self {
            current_scene: 0,
            scenes,
            backgrounds,
            time: 0.,
            switching: true,
        }
    }

    fn empty() -> Self {
        Scenes {
            current_scene: 0,
            scenes: vec![SceneConfig::default()],
            backgrounds: vec![],
            time: 0.,
            switching: false,
        }
    }

    fn is_switching(&self) -> bool {
        self.switching && self.display_time() > 0
    }

    fn tick(&mut self, delta: f32) {
        self.time += delta;
    }

    fn needs_reloading(&self) -> bool {
        self.is_switching() && self.time > self.display_time() as f32
    }

    fn draw(&self, rect: Vec2) {
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

    fn input_data_path(&self) -> Option<String> {
        self.scenes[self.current_scene].input_data_path.clone()
    }

    fn legend(&self) -> Option<Legend> {
        self.scenes[self.current_scene].legend.clone()
    }

    fn next(&mut self) {
        self.time = 0.;
        self.current_scene += 1;
        if self.current_scene == self.scenes.len() {
            self.current_scene = 0;
        }
    }

    fn toggle_switching(&mut self) -> bool {
        self.time = 0.;
        self.switching = !self.switching;
        self.switching
    }
}

#[derive(DeJson)]
pub struct FishSpeed {
    pub x: f32,
    pub y: f32,
}

impl From<&FishSpeed> for Vec2 {
    fn from(serializable: &FishSpeed) -> Vec2 {
        vec2(serializable.x, serializable.y)
    }
}

#[derive(DeJson)]
pub struct FishArea {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl From<&FishArea> for Rect {
    fn from(serializable: &FishArea) -> Rect {
        Rect {
            x: serializable.x,
            y: serializable.y,
            w: serializable.w,
            h: serializable.h,
        }
    }
}

struct Resources {
    config: Config,
    input_data: InputData,
    backgrounds: Vec<Texture2D>,
    bubble_texture: Texture2D,
    fish_textures: HashMap<String, Texture2D>,
}

impl Resources {
    async fn new() -> Result<Resources, macroquad::prelude::FileError> {
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

    async fn load() {
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

#[derive(Clone, DeJson, Debug)]
#[nserde(default)]
pub struct FishConfig {
    pub texture: String,
    pub size: f32,
    pub size_randomness: f32,
    pub movement: Movement,
    pub bubbles: u32,
    #[nserde(proxy = "FishSpeed")]
    pub speed: Vec2,
    #[nserde(proxy = "FishSpeed")]
    pub speed_randomness: Vec2,
    pub collision_aversion: f32,
    #[nserde(proxy = "FishArea")]
    pub area: Rect,
}

impl Default for FishConfig {
    fn default() -> Self {
        Self {
            texture: "ferris.png".to_string(),
            size: 7.,
            size_randomness: 0.5,
            movement: Movement::Accelerating,
            bubbles: 25,
            speed: vec2(15., 7.),
            speed_randomness: vec2(0.5, 0.5),
            collision_aversion: 0.90,
            area: Rect {
                x: 5.,
                y: 5.,
                w: 90.,
                h: 52.5,
            },
        }
    }
}

impl FishConfig {
    fn randomized_size(&self) -> f32 {
        self.size - self.size * rand::gen_range(0.0, self.size_randomness)
    }

    fn randomized_speed(&self) -> Vec2 {
        let random_speed = vec2(
            rand::gen_range(0., self.speed_randomness.x),
            rand::gen_range(0., self.speed_randomness.y),
        );
        self.speed - self.speed * random_speed
    }

    fn randomized_bubble_amount(&self) -> u32 {
        rand::gen_range(0, 25)
    }
}

#[derive(Clone, DeJson, Debug)]
#[nserde(default)]
pub struct SceneConfig {
    pub input_data_path: Option<String>,
    pub display_time: u32,
    pub background: Option<usize>,
    pub legend: Option<Legend>,
}

impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            input_data_path: None,
            display_time: 30,
            background: None,
            legend: None,
        }
    }
}

impl SceneConfig {
    fn new(input_data_path: Option<String>, display_time: u32) -> Self {
        Self {
            input_data_path,
            display_time,
            background: None,
            legend: None,
        }
    }
}

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
    async fn load() -> Self {
        let json = load_string("config.json")
            .await
            .unwrap_or_else(|_| "{}".to_string());
        DeJson::deserialize_json(&json).expect("Failed parsing config")
    }

    async fn background_textures(&self) -> Vec<Texture2D> {
        let background_futures = self
            .backgrounds
            .iter()
            .map(|background| load_texture(background));
        join_all(background_futures)
            .await
            .into_iter()
            .flatten()
            .collect()
    }
}

#[derive(Clone, DeJson)]
#[nserde(default)]
pub struct FishData {
    pub fish: String,
    pub size: f32,
    pub speed: f32,
    pub bubbles: f32,
}

impl Default for FishData {
    fn default() -> FishData {
        FishData {
            fish: "clownfish".to_string(),
            size: 1.0,
            speed: 1.0,
            bubbles: 1.0,
        }
    }
}

#[derive(Clone, DeJson, Debug)]
pub struct FishLegend {
    pub fish: String,
    pub description: String,
}

#[derive(Clone, DeJson, Debug)]
pub struct Legend {
    pub description: String,
    pub fish_legends: Vec<FishLegend>,
}

#[derive(Clone, DeJson, Default)]
pub struct InputData {
    pub legend: Option<Legend>,
    pub school: Vec<FishData>,
}

impl InputData {
    async fn load(path: String) -> Self {
        let json = if Self::is_url(&path) {
            Self::load_url(path).await
        } else {
            load_string(path.as_str()).await.ok()
        };
        DeJson::deserialize_json(&json.unwrap_or_else(|| "{}".to_string())).unwrap_or_default()
    }

    fn is_url(path: &str) -> bool {
        path.starts_with("http://") || path.starts_with("https://")
    }

    async fn load_url(path: String) -> Option<String> {
        let mut request = RequestBuilder::new(path.as_str()).send();
        loop {
            if let Some(result) = request.try_recv() {
                return match result {
                    Ok(data) => Some(data),
                    Err(error) => {
                        error!("Error reading inputdata: {}", error);
                        None
                    }
                };
            }
            next_frame().await;
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Rusty Aquarium".to_owned(),
        fullscreen: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    const SCR_W: f32 = 100.0;
    const SCR_H: f32 = 62.5;

    macroquad::file::set_pc_assets_folder("assets");
    let crt_render_target = render_target(screen_width() as u32, screen_height() as u32);
    crt_render_target.texture.set_filter(FilterMode::Linear);
    let water_render_target = render_target(screen_width() as u32, screen_height() as u32);
    water_render_target.texture.set_filter(FilterMode::Linear);
    let water_material = load_material(
        shaders::water_wave::VERTEX,
        shaders::water_wave::FRAGMENT,
        Default::default(),
    )
    .unwrap();
    let crt_material = load_material(
        shaders::crt::VERTEX,
        shaders::crt::FRAGMENT,
        Default::default(),
    )
    .unwrap();
    let mut shader_activated = false;

    let mut fish_tank = FishTank::new();
    let mut show_text = ShowText::empty();
    let mut show_help = ShowHelp::new();

    loop {
        if !fish_tank.loaded {
            Resources::load().await;
            fish_tank.add_resources();
        }

        #[cfg(not(target_arch = "wasm32"))]
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Q) {
            return;
        }
        if is_key_pressed(KeyCode::Left) || is_mouse_button_pressed(MouseButton::Middle) {
            shader_activated = !shader_activated;
            show_text = if shader_activated {
                ShowText::new("Activated shader")
            } else {
                ShowText::new("Disabled shader")
            };
        }
        if is_key_pressed(KeyCode::Right) || is_mouse_button_pressed(MouseButton::Left) {
            fish_tank.next_scene();
            show_text = ShowText::new("Next scene");
        }
        if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Right) {
            show_text = if fish_tank.toggle_switching_scenes() {
                ShowText::new("Switching scenes")
            } else {
                ShowText::new("Scene locked")
            };
        }
        if is_key_pressed(KeyCode::Enter) {
            fish_tank.repopulate();
        }
        if is_key_pressed(KeyCode::Up) {
            fish_tank.add_fish();
        }
        if is_key_pressed(KeyCode::Down) {
            fish_tank.remove_fish();
        }
        if is_key_pressed(KeyCode::C) {
            show_text = ShowText::new("Updating config...");
            let config = Config::load().await;
            fish_tank.update_config(config);
        }
        if is_key_pressed(KeyCode::D) {
            show_text = ShowText::new("Reloading data...");
            fish_tank.reload_data();
        }
        if is_key_pressed(KeyCode::L) || is_key_pressed(KeyCode::I) {
            fish_tank.toggle_legend();
        }
        if is_key_pressed(KeyCode::H) {
            show_help.toggle_show();
        }

        // Update fish positions
        let delta = get_frame_time();

        fish_tank.tick(delta);

        // build camera with following coordinate system:
        // (0., 0)     .... (SCR_W, 0.)
        // (0., SCR_H) .... (SCR_W, SCR_H)
        set_camera(&Camera2D {
            zoom: vec2(1. / SCR_W * 2., -1. / SCR_H * 2.),
            target: vec2(SCR_W / 2., SCR_H / 2.),
            render_target: Some(water_render_target),
            ..Default::default()
        });
        clear_background(DARKBLUE);

        // Draw fish_tank
        fish_tank.draw(vec2(SCR_W, SCR_H));

        // Draw texture with water shader
        if shader_activated {
            set_camera(&Camera2D {
                zoom: vec2(1. / SCR_W * 2., -1. / SCR_H * 2.),
                target: vec2(SCR_W / 2., SCR_H / 2.),
                render_target: Some(crt_render_target),
                ..Default::default()
            });
            clear_background(DARKBLUE);
            gl_use_material(water_material);

            draw_texture_ex(
                water_render_target.texture,
                0.,
                0.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    flip_y: true,
                    ..Default::default()
                },
            );

            // Draw texture to screen with crt shader
            set_default_camera();
            clear_background(DARKBLUE);

            gl_use_material(crt_material);

            draw_texture_ex(
                crt_render_target.texture,
                0.,
                0.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    flip_y: true,
                    ..Default::default()
                },
            );
            gl_use_default_material();
        } else {
            set_default_camera();
            clear_background(DARKBLUE);

            draw_texture_ex(
                water_render_target.texture,
                0.,
                0.,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(screen_width(), screen_height())),
                    flip_y: true,
                    ..Default::default()
                },
            );
        }

        show_text.draw(delta);
        fish_tank.draw_legend();
        show_help.draw();

        next_frame().await
    }
}
