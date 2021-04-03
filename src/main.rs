use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;
use macroquad_particles::{ Emitter, EmitterConfig, ParticleMaterial };
use futures::future::join_all;
use nanoserde::{DeJson};
use std::collections::HashMap;
use std::iter::FromIterator;

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

    fn move_position(&mut self, delta: f32, motion: Motion) -> Motion {
        //debug!("x: {} y: {} d: {}", self.position.x, self.position.y, delta);

        let position = if motion.idle {
            motion.position
        } else {
            motion.position + motion.speed * delta
        };

        //debug!("rotation: {} new_pos: {} old_pos: {}", rotation, new_position, self.motion.position);
        Motion {
            position: position,
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
            self.idle = self.idle ^ (Self::random_percent() < Movement::CHANCE_IDLE_END);
        } else {
            self.idle = self.idle ^ (Self::random_percent() < Movement::CHANCE_IDLE_START);
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

    fn change_acceleration_randomly(&mut self, change_chance: Vec2) {
        if Self::random_percent() < change_chance.x {
            self.acceleration.x *= -1.;
        }
        if Self::random_percent() < change_chance.y {
            self.acceleration.y *= -1.;
        }
    }

    fn clamp(&mut self, bounding_box: Rect) {
        self.position = self.position
            .max(bounding_box.point())
            .min(vec2(bounding_box.right(), bounding_box.bottom()));
    }

    fn random_percent() -> f32 {
        rand::gen_range(0., 100.)
    }
}

#[derive(Copy, Clone, DeJson)]
pub enum Movement {
    SingleSpeed,
    Accelerating,
    AcceleratingEdgeIdling,
    Crab,
    Random,
}

impl Default for Movement {
    fn default() -> Self { Movement::Accelerating }
}

impl Movement {
    const DIRECTION_CHANGE_CHANCE: Vec2 = Vec2 { x: 2.5, y: 5. };
    const CHANCE_IDLE_START: f32 = 0.05;
    const CHANCE_IDLE_END: f32 = 0.75;

    fn tick(&mut self, motion: Motion, bounding_box: Rect) -> Motion {
        match self {
            Self::SingleSpeed => Self::tick_single_speed(motion, bounding_box),
            Self::Accelerating => Self::tick_accelerating(motion, bounding_box),
            Self::AcceleratingEdgeIdling => Self::tick_accelerating_edge_idling(motion, bounding_box),
            Self::Crab => Self::tick_crab(motion, bounding_box),
            Self::Random => Self::tick_random(motion, bounding_box),
        }
    }

    fn random() -> Self {
        *vec![Self::SingleSpeed, Self::Accelerating, Self::AcceleratingEdgeIdling, Self::Random].choose().unwrap()
    }

    fn tick_single_speed(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(Self::DIRECTION_CHANGE_CHANCE);
        motion.clamp(bounding_box);
        motion.rotate();
        motion
    }

    fn tick_accelerating(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.accelerate();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(Self::DIRECTION_CHANGE_CHANCE);
        motion.clamp(bounding_box);
        motion.rotate();
        motion
    }

    fn tick_accelerating_edge_idling(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.accelerate();
        motion.change_direction_vertically(bounding_box);
        motion.change_acceleration_randomly(Self::DIRECTION_CHANGE_CHANCE);
        motion.clamp(bounding_box);
        motion.rotate();
        motion
    }

    fn tick_crab(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.accelerate();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(Self::DIRECTION_CHANGE_CHANCE * 5.);
        motion.clamp(bounding_box);
        motion
    }

    fn tick_random(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.accelerate();
        motion.random_idling();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(Self::DIRECTION_CHANGE_CHANCE);
        motion.clamp(bounding_box);
        motion.rotate();
        motion
    }
}

pub struct Fish {
    motion: Motion,
    movement: Movement,
    size: Vec2,
    //bounding_box: Rect,
    bounding_box_adjusted: Rect,
    texture: Texture2D,
    emitter: Emitter,
    bubbles: bool,
}
impl Fish {
    const SPRITE_BUBBLE: &'static str = "assets/bubble.png";
    //const SPRITE_YELLOWSUBMARINE: &'static str = "assets/yellowsubmarine.png";

    fn new(
        fish_size: f32,
        max_speed: Vec2,
        bounding_box: Rect,
        movement: Movement,
        texture: Texture2D,
        bubble_texture: Texture2D,
        bubbles: bool) -> Fish {
        let fish_height = fish_size / (texture.width() / texture.height());
        let size = vec2(fish_size, fish_height);
        let bbox_adjusted = Self::adjust_bounding_box(bounding_box, size);
        Fish {
            motion: Motion {
                position: Self::random_start_position(bbox_adjusted),
                speed: Self::random_start_speed(max_speed),
                max_speed: max_speed,
                acceleration: Self::random_acceleration(),
                rotation: 0.,
                idle: false,
            },
            size: size,
            //bounding_box: bounding_box,
            bounding_box_adjusted: bbox_adjusted,
            movement: movement,
            texture: texture,
            emitter: Emitter::new(EmitterConfig {
                emitting: true,
                amount: 25,
                lifetime: 1.4,
                lifetime_randomness: 0.9,
                size: 0.55,
                size_randomness: 0.9,
                explosiveness: 0.9,
                initial_velocity: 5.0,
                initial_velocity_randomness: 0.8,
                initial_direction_spread: 0.5,
                gravity: vec2(0.0, -5.0),
                texture: Some(bubble_texture),
                material: Some(water_particle_shader::material()),
                ..Default::default()
            }),
            bubbles: bubbles,
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

    fn random_start_speed(max_speed: Vec2) -> Vec2 {
        vec2(
            rand::gen_range(-max_speed.x, max_speed.x),
            rand::gen_range(-max_speed.y, max_speed.y),
        )
    }

    fn random_acceleration() -> Vec2 {
        vec2(
            rand::gen_range(0.1, 0.2),
            rand::gen_range(0.1, 0.2),
        )
    }

    fn tick(&mut self, delta: f32) {
        let motion = self.movement.tick(self.motion, self.bounding_box_adjusted);
        self.motion = self.motion.move_position(delta, motion);
    }

    fn swims_right(&self) -> bool {
        return self.motion.speed.x >= 0.;
    }

    fn emit_position(&self) -> Vec2 {
        return self.motion.position
            + if !self.swims_right() { vec2(self.size.x, 0.) } else { vec2(0., 0.) }
            + vec2(0., self.size.y / 2.);
    }

    fn emit(&mut self) {
        match self.movement {
            Movement::Crab => (),
            _ => if self.bubbles {
                self.emitter.draw(self.emit_position())
            },
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
    fish_keys: Vec<String>,
    config: Config,
    school: Vec<FishData>,
    bubble_texture: Texture2D,
    fish_textures: HashMap<String, Texture2D>,
    //input_data: InputData,
    time_since_reload: f32,
}

impl FishTank {
    fn new(bubble_texture: Texture2D, fish_textures: HashMap<String, Texture2D>, config: Config, input_data: InputData) -> Self {
        Self {
            fishes: Vec::new(),
            fish_keys: Vec::from_iter(config.fishes.keys().cloned()),
            config: config,
            school: input_data.school,
            bubble_texture: bubble_texture,
            fish_textures: fish_textures,
            time_since_reload: 0.,
        }
    }

    async fn reload_config(&mut self, delta: f32) {
        if self.config.data_reload_time == 0 {
            return;
        }
        self.time_since_reload += delta;
        if self.time_since_reload > self.config.data_reload_time as f32 {
            let data = InputData::load().await;
            self.update_data(data);
            self.time_since_reload = 0.;
        }
    }

    fn update_data(&mut self, input_data: InputData) {
        self.school = input_data.school;
        self.repopulate();
    }

    fn tick(&mut self, delta: f32) {
        for fish in self.fishes.iter_mut() {
            fish.tick(delta);
        }
    }

    fn draw(&mut self) {
        for fish in self.fishes.iter_mut() {
            fish.draw();
        }
    }

    fn populate(&mut self) {
        for fish_data in self.school.iter() {
            let fish = self.create_fish(fish_data);
            self.fishes.push(fish);
        }
    }

    fn reset(&mut self) {
        self.fishes.clear();
    }

    fn repopulate(&mut self) {
        if self.fishes.len() >= 1 {
            self.reset();
            self.populate();
        }
    }

    fn add_fish(&mut self) {
        let fish = self.random_fish();
        //debug!("size: {:?}", fish.size);
        //debug!("speed: {:?}", fish.motion.speed);
        //debug!("bubbles: {:?}", fish.bubbles);
        //debug!("---");
        self.fishes.push(fish);
    }

    fn remove_fish(&mut self) {
        // Don't remove Ferris
        if self.fishes.len() > 0 {
            self.fishes.pop();
        }
    }

    fn random_fish_config(&self) -> &FishConfig {
        let fish_key = self.fish_keys.choose().unwrap();
        return self.config.fishes.get(fish_key).unwrap();
    }

    fn random_fish(&self) -> Fish {
        let fish_config = self.random_fish_config();
        Fish::new(
            fish_config.randomized_size(),
            fish_config.randomized_speed(),
            fish_config.area,
            fish_config.movement,
            *self.fish_textures.get(&fish_config.texture).unwrap(),
            self.bubble_texture,
            fish_config.bubbles,
        )
    }

    fn create_fish(&self, fish_data: &FishData) -> Fish {
        let fish_config = self.config.fishes.get(&fish_data.fish).unwrap();
        Fish::new(
            fish_config.size * fish_data.size,
            fish_config.speed * fish_data.speed,
            fish_config.area,
            fish_config.movement,
            *self.fish_textures.get(&fish_config.texture).unwrap(),
            self.bubble_texture,
            fish_config.bubbles,
        )
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
            text: text,
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
            draw_text(
                self.text,
                self.x,
                self.y,
                40.,
                WHITE,
                );
        }
    }
}

struct ShowBackground {
    background: usize,
    backgrounds: Vec<Texture2D>,
    time: f32,
    background_switch_time: f32,
    switching_backgrounds: bool,
}

impl ShowBackground {
    fn new(background_switch_time: u32, backgrounds: Vec<Texture2D>) -> Self {
        Self {
            background: 0,
            backgrounds: backgrounds,
            time: 0.,
            background_switch_time: background_switch_time as f32,
            switching_backgrounds: true,
        }
    }

    fn draw(&mut self, delta: f32, w: f32, h: f32) {
        self.time += delta;

        if self.time > self.background_switch_time && self.switching_backgrounds {
            self.next();
        }

        draw_texture_ex(
            self.backgrounds[self.background],
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                ..Default::default()
            },
        );
    }

    fn next(&mut self) {
        self.time = 0.;
        self.background += 1;
        if self.background == self.backgrounds.len() {
            self.background = 0;
        }
    }

    fn toggle_switching_backgrounds(&mut self) {
        self.switching_backgrounds = !self.switching_backgrounds;
        self.time  = 0.;
    }
}

#[derive(DeJson)]
pub struct FishSpeed {
    pub x: f32,
    pub y: f32,
}

impl From<&FishSpeed> for Vec2 {
    fn from(serializable: &FishSpeed) -> Vec2 {
        Vec2 {
            x: serializable.x,
            y: serializable.y,
        }
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

#[derive(Clone, DeJson)]
#[nserde(default)]
pub struct FishConfig {
    pub texture: String,
    pub size: f32,
    pub size_randomness: f32,
    pub movement: Movement,
    pub bubbles: bool,
    #[nserde(proxy = "FishSpeed")]
    pub speed: Vec2,
    #[nserde(proxy = "FishSpeed")]
    pub speed_randomness: Vec2,
    #[nserde(proxy = "FishArea")]
    pub area: Rect,
}

impl Default for FishConfig {
    fn default() -> FishConfig {
        FishConfig {
            texture: "assets/ferris.png".to_string(),
            size: 7.,
            size_randomness: 0.5,
            movement: Movement::Accelerating,
            bubbles: true,
            speed: vec2(15., 7.),
            speed_randomness: vec2(0.5, 0.5),
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
        return self.size - self.size * rand::gen_range(0.0, self.size_randomness);
    }

    fn randomized_speed(&self) -> Vec2 {
        let random_speed = vec2(
            rand::gen_range(0., self.speed_randomness.x),
            rand::gen_range(0., self.speed_randomness.y));
        return self.speed - self.speed * random_speed;
    }
}

#[derive(DeJson)]
#[nserde(default)]
pub struct Config {
    pub data_reload_time: u32,
    pub background_switch_time: u32,
    pub backgrounds: Vec<String>,
    pub fishes: HashMap<String, FishConfig>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            data_reload_time: 0,
            background_switch_time: 0,
            backgrounds: vec![],
            fishes: HashMap::new(),
        }
    }
}

impl Config {
    async fn load() -> Self {
        let json = load_string("assets/config.json").await.unwrap();
        return DeJson::deserialize_json(&json).unwrap();
    }

    async fn background_textures(&self) -> Vec<Texture2D> {
        let background_futures = self.backgrounds.iter().map( |background| { load_texture(background) });
        return join_all(background_futures).await;
    }
}

#[derive(Clone, DeJson)]
pub struct FishData {
    pub fish: String,
    pub size: f32,
    pub speed: f32,
}

#[derive(Clone, DeJson)]
pub struct InputData {
    pub school: Vec<FishData>,
}

impl InputData {
    async fn load() -> Self {
        let json = load_string("assets/inputdata.json").await.unwrap();
        return DeJson::deserialize_json(&json).unwrap();
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Rusty Aquarium".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    const SCR_W: f32 = 100.0;
    const SCR_H: f32 = 62.5;

    let config = Config::load().await;
    let input_data = InputData::load().await;

    let bubble_texture: Texture2D = load_texture(Fish::SPRITE_BUBBLE).await;
    //let submarine: Texture2D = load_texture(Fish::SPRITE_YELLOWSUBMARINE).await;

    let crt_render_target = render_target(screen_width() as u32, screen_height() as u32);
    set_texture_filter(crt_render_target.texture, FilterMode::Linear);
    let water_render_target = render_target(screen_width() as u32, screen_height() as u32);
    set_texture_filter(water_render_target.texture, FilterMode::Linear);
    let water_material = load_material(water_wave_shader::VERTEX, water_wave_shader::FRAGMENT, Default::default()).unwrap();
    let crt_material = load_material(crt_shader::VERTEX, crt_shader::FRAGMENT, Default::default()).unwrap();
    let mut shader_activated = false;
    let mut background = ShowBackground::new(config.background_switch_time, config.background_textures().await);
    let mut fish_textures = HashMap::new();
    for (_key, fish) in config.fishes.iter() {
        fish_textures.insert(fish.texture.clone(), load_texture(&fish.texture).await);
    }
    let mut fish_tank = FishTank::new(bubble_texture, fish_textures, config, input_data);
    let mut show_text: ShowText = ShowText::empty();

    fish_tank.populate();

    loop {
        if is_key_pressed(KeyCode::Escape) {
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
            background.next();
            show_text = ShowText::new("Next background");
        }
        if is_key_pressed(KeyCode::Space) || is_mouse_button_pressed(MouseButton::Right) {
            background.toggle_switching_backgrounds();
            show_text = if background.switching_backgrounds {
                ShowText::new("Switching backgrounds")
            } else {
                ShowText::new("Background locked")
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

        // Update fish positions
        let delta = get_frame_time();

        fish_tank.reload_config(delta).await;

        fish_tank.tick(delta);

        // build camera with following coordinate system:
        // (0., 0)     .... (SCR_W, 0.)
        // (0., SCR_H) .... (SCR_W, SCR_H)
        set_camera(Camera2D {
            zoom: vec2(1. / SCR_W * 2., -1. / SCR_H * 2.),
            target: vec2(SCR_W / 2., SCR_H / 2.),
            render_target: Some(water_render_target),
            ..Default::default()
        });
        clear_background(DARKBLUE);

        // Draw background
        background.draw(delta, SCR_W, SCR_H);

        // Draw fish_tank
        fish_tank.draw();


        // Draw texture with water shader
        if shader_activated {
            set_camera(Camera2D {
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

        next_frame().await
    }
}

mod crt_shader {
    pub const FRAGMENT: &'static str = r#"#version 100
        precision lowp float;
        varying vec4 color;
        varying vec2 uv;

        uniform sampler2D Texture;
        // https://www.shadertoy.com/view/XtlSD7
        vec2 CRTCurveUV(vec2 uv)
        {
            uv = uv * 2.0 - 1.0;
            vec2 offset = abs( uv.yx ) / vec2( 6.0, 4.0 );
            uv = uv + uv * offset * offset;
            uv = uv * 0.5 + 0.5;
            return uv;
        }
        void DrawVignette( inout vec3 color, vec2 uv )
        {
            float vignette = uv.x * uv.y * ( 1.0 - uv.x ) * ( 1.0 - uv.y );
            vignette = clamp( pow( 16.0 * vignette, 0.3 ), 0.0, 1.0 );
            color *= vignette;
        }
        void DrawScanline( inout vec3 color, vec2 uv )
        {
            float iTime = 0.1;
            float scanline = clamp( 0.95 + 0.05 * cos( 3.14 * ( uv.y + 0.008 * iTime ) * 240.0 * 1.0 ), 0.0, 1.0 );
            float grille = 0.85 + 0.15 * clamp( 1.5 * cos( 3.14 * uv.x * 640.0 * 1.0 ), 0.0, 1.0 );
            color *= scanline * grille * 1.2;
        }
        void main() {

            vec2 crtUV = CRTCurveUV(uv);

            vec3 res = texture2D(Texture, uv).rgb * color.rgb;

            if (crtUV.x < 0.0 || crtUV.x > 1.0 || crtUV.y < 0.0 || crtUV.y > 1.0)
            {
                res = vec3(0.0, 0.0, 0.0);
            }
            DrawVignette(res, crtUV);
            DrawScanline(res, uv);
            gl_FragColor = vec4(res, 1.0);
        }
    "#;

    pub const VERTEX: &'static str = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;
        attribute vec4 color0;
        varying lowp vec2 uv;
        varying lowp vec4 color;
        uniform mat4 Model;
        uniform mat4 Projection;
        void main() {
            gl_Position = Projection * Model * vec4(position, 1);
            color = color0 / 255.0;
            uv = texcoord;
        }
    "#;
}

mod water_wave_shader {
    pub const FRAGMENT: &'static str = r#"#version 100
        precision lowp float;

        varying vec2 uv;
        varying vec2 uv1;

        uniform vec4 _Time;
        uniform sampler2D Texture;
        uniform sampler2D _ScreenTexture;

        #define amp 0.02

        void main() {
            vec2 p = uv;
            vec2 h = uv1 * 0.003; // Size of waves
            float time = _Time.x;

            h.x += sin(h.y * 15. + time * 2.) / 30.;
            h.y += cos(h.x * 10. + time * 2.) / 30.;

            p.x += sin((h.y + h.x) * 15. + time * 2.) / (400. + (10. * sin(time)));
            p.y += cos((h.y + h.x) * 15. + time * 2.) / (400. + (10. * sin(time)));

            vec3 res = texture2D(Texture, p).rgb * vec3(0.8, 0.8, 0.9) + vec3(0.0, 0.0, 0.04 * sin(h.y * 15. + time * 2.)) * cos(h.x * 10. + time * 2.);

            gl_FragColor = vec4(res, 1.0);
        }
    "#;

    pub const VERTEX: &'static str = r#"#version 100
        attribute vec3 position;
        attribute vec2 texcoord;

        varying lowp vec4 color;
        varying lowp vec2 uv;
        varying lowp vec2 uv1;

        uniform mat4 Model;
        uniform mat4 Projection;

        void main() {
            vec4 res = Projection * Model * vec4(position, 1);

            uv = res.xy / 2.0 + vec2(0.5, 0.5);
            uv1 = position.xy;

            gl_Position = res;
        }
    "#;
}

mod water_particle_shader {
    use super::*;

    pub fn material() -> ParticleMaterial {
        return ParticleMaterial::new(water_particle_shader::VERTEX, water_particle_shader::PARTICLE);
    }

    pub const VERTEX: &'static str = r#"#version 100
        #define DEF_VERTEX_ATTRIBUTES
        #include "particles.glsl"

        varying lowp vec2 texcoord;
        varying lowp vec4 particle_data;

        void main() {
            gl_Position = particle_transform_vertex();
            texcoord = particle_transform_uv();
            particle_data = in_attr_inst_data;
        }
    "#;

    pub const PARTICLE: &'static str = r#"#version 100
        #include "particles.glsl"

        precision lowp float;
        varying lowp vec2 texcoord;
        varying lowp vec4 particle_data;

        uniform sampler2D texture;

        void main() {
            // particle_lifetime is 0..1 value with 0 at the beginning of particle life and 1 just before particle removal
            float fade_during_lifetime = 0.5 + (1.0 - particle_lifetime(particle_data));

            gl_FragColor = texture2D(texture, texcoord) * vec4(1., 1., 1., 0.2) * fade_during_lifetime;
        }
    "#;
}
