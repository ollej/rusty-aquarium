use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;
use macroquad_particles::{ Emitter, EmitterConfig, ParticleMaterial };

fn window_conf() -> Conf {
    Conf {
        window_title: "Rusty Aquarium".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
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

    fn move_position(&mut self, delta: f32, motion: Motion) -> Motion {
        //debug!("x: {} y: {} d: {}", self.position.x, self.position.y, delta);

        let position = if motion.idle {
            motion.position
        } else {
            motion.position + motion.speed * delta
        };

        //debug!("rotation: {} new_pos: {} old_pos: {}", rotation, new_position, self.motion.position);
        return Motion {
            position: position,
            speed: motion.speed,
            max_speed: motion.max_speed,
            acceleration: motion.acceleration,
            rotation: motion.rotation,
            idle: motion.idle,
        }
    }

    fn rotate(&mut self) {
        self.rotation = (self.speed.y / self.max_speed.y).abs() * Motion::MAX_ROTATION;
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
            self.idle = self.idle ^ (Motion::random_percent() < Movement::CHANCE_IDLE_END);
        } else {
            self.idle = self.idle ^ (Motion::random_percent() < Movement::CHANCE_IDLE_START);
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
        if Motion::random_percent() < change_chance.x {
            self.acceleration.x *= -1.;
        }
        if Motion::random_percent() < change_chance.y {
            self.acceleration.y *= -1.;
        }
    }

    fn clamp(&mut self, bounding_box: Rect) {
        self.position = self.position
            .max(bounding_box.point())
            .min(vec2(bounding_box.right(), bounding_box.bottom()));
    }

    fn random_percent() -> f32 {
        return rand::gen_range(0., 100.);
    }
}

#[derive(Copy, Clone)]
pub enum Movement {
    SingleSpeed,
    Accelerating,
    AcceleratingEdgeIdling,
    Crab,
    Random,
}
impl Movement {
    const DIRECTION_CHANGE_CHANCE: Vec2 = Vec2 { x: 2.5, y: 5. };
    const CHANCE_IDLE_START: f32 = 0.05;
    const CHANCE_IDLE_END: f32 = 0.75;

    fn tick(&mut self, motion: Motion, bounding_box: Rect) -> Motion {
        return match self {
            Movement::SingleSpeed => Movement::tick_single_speed(motion, bounding_box),
            Movement::Accelerating => Movement::tick_accelerating(motion, bounding_box),
            Movement::AcceleratingEdgeIdling => Movement::tick_accelerating_edge_idling(motion, bounding_box),
            Movement::Crab => Movement::tick_crab(motion, bounding_box),
            Movement::Random => Movement::tick_random(motion, bounding_box),
        }
    }

    fn random() -> Movement {
        return *vec![Movement::SingleSpeed, Movement::Accelerating, Movement::AcceleratingEdgeIdling, Movement::Random].choose().unwrap();
    }

    fn tick_single_speed(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(Movement::DIRECTION_CHANGE_CHANCE);
        motion.clamp(bounding_box);
        motion.rotate();
        return motion;
    }

    fn tick_accelerating(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.accelerate();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(Movement::DIRECTION_CHANGE_CHANCE);
        motion.clamp(bounding_box);
        motion.rotate();
        return motion;
    }

    fn tick_accelerating_edge_idling(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.accelerate();
        motion.change_direction_vertically(bounding_box);
        motion.change_acceleration_randomly(Movement::DIRECTION_CHANGE_CHANCE);
        motion.clamp(bounding_box);
        motion.rotate();
        return motion;
    }

    fn tick_crab(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.accelerate();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(Movement::DIRECTION_CHANGE_CHANCE * 5.);
        motion.clamp(bounding_box);
        return motion;
    }

    fn tick_random(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.accelerate();
        motion.random_idling();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_acceleration_randomly(Movement::DIRECTION_CHANGE_CHANCE);
        motion.clamp(bounding_box);
        motion.rotate();
        return motion;
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
}
impl Fish {
    const SPRITE_CLOWNFISH: &'static str = "assets/clownfish.png";
    const SPRITE_ANGELFISH: &'static str = "assets/angelfish.png";
    const SPRITE_GOLDFISH: &'static str = "assets/goldfish.png";
    const SPRITE_YELLOWFISH: &'static str = "assets/yellowfish.png";
    const SPRITE_SEAHORSE: &'static str = "assets/seahorse.png";
    const SPRITE_ROYALGRAMMA: &'static str = "assets/royalgramma.png";
    const SPRITE_BUTTERFLYFISH: &'static str = "assets/butterflyfish.png";
    const SPRITE_LIONFISH: &'static str = "assets/lionfish.png";
    const SPRITE_TURTLE: &'static str = "assets/turtle.png";
    const SPRITE_NEONTETRA: &'static str = "assets/neontetra.png";
    const SPRITE_YELLOWANGELFISH: &'static str = "assets/yellowangelfish.png";
    const SPRITE_ZEBRAFISH: &'static str = "assets/zebrafish.png";
    const SPRITE_CRAB: &'static str = "assets/ferris.png";
    const SPRITE_BUBBLE: &'static str = "assets/bubble.png";
    //const SPRITE_YELLOWSUBMARINE: &'static str = "assets/yellowsubmarine.png";
    const MAX_POSITION: Vec2 = Vec2 { x: 5., y: 5. };
    const MIN_POSITION: Vec2 = Vec2 { x: 5., y: 5. };
    const DEFAULT_SPRITE_WIDTH: f32 = 7.;

    fn new(
        fish_size: f32,
        max_speed: Vec2,
        bounding_box: Rect,
        movement: Movement,
        texture: Texture2D,
        bubble_texture: Texture2D) -> Fish {
        let fish_height = fish_size / (texture.width() / texture.height());
        let size = vec2(fish_size, fish_height);
        let bbox_adjusted = Fish::adjust_bounding_box(bounding_box, size);
        Fish {
            motion: Motion {
                position: Fish::random_start_position(bbox_adjusted),
                speed: Fish::random_start_speed(max_speed),
                max_speed: max_speed,
                acceleration: Fish::random_acceleration(),
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
        }
    }

    fn adjust_bounding_box(bounding_box: Rect, size: Vec2) -> Rect {
        return Rect {
            x: bounding_box.x,
            y: bounding_box.y,
            w: bounding_box.w - size.x,
            h: bounding_box.h - size.y,
        };
    }

    fn random_start_position(bounding_box: Rect) -> Vec2 {
        return vec2(
            rand::gen_range(bounding_box.x, bounding_box.right()),
            rand::gen_range(bounding_box.y, bounding_box.bottom()));
    }

    fn random_start_speed(max_speed: Vec2) -> Vec2 {
        return vec2(
            rand::gen_range(-max_speed.x, max_speed.x),
            rand::gen_range(-max_speed.y, max_speed.y),
        );
    }

    fn random_acceleration() -> Vec2 {
        return vec2(
            rand::gen_range(0.1, 0.2),
            rand::gen_range(0.1, 0.2),
        );
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
            _ => self.emitter.draw(self.emit_position()),
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
    bounding_box: Rect,
    ferris_texture: Texture2D,
    bubble_texture: Texture2D,
    fish_textures: Vec<Texture2D>,
}

impl FishTank {
    fn new(screen_width: f32, screen_height: f32, ferris_texture: Texture2D, bubble_texture: Texture2D, fish_textures: Vec<Texture2D>) -> Self {
        Self {
            fishes: Vec::new(),
            ferris_texture: ferris_texture,
            bubble_texture: bubble_texture,
            fish_textures: fish_textures,
            bounding_box: Self::default_bounding_box(screen_width, screen_height),
        }
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

    fn default_bounding_box(width: f32, height: f32) -> Rect {
        Rect {
            x: Fish::MIN_POSITION.x,
            y: Fish::MIN_POSITION.y,
            w: width - Fish::MAX_POSITION.x - Fish::MIN_POSITION.x,
            h: height - Fish::MAX_POSITION.y - Fish::MIN_POSITION.y,
        }
    }

    fn populate(&mut self, count: usize) {
        // Only add Ferris if fishes is empty
        if self.fishes.len() == 0 {
            self.fishes.push(self.ferris());
        }
        for _ in 0..count {
            self.add_fish();
        }
    }

    fn reset(&mut self) {
        self.fishes.truncate(1); // Keep Ferris
    }

    fn repopulate(&mut self) {
        let count = self.fish_count();
        if count >= 1 {
            self.reset();
            self.populate(count);
        }
    }

    fn fish_count(&self) -> usize {
        return self.fishes.len() - 1; // Skip Ferris
    }

    fn add_fish(&mut self) {
        self.fishes.push(self.fish());
    }

    fn remove_fish(&mut self) {
        // Don't remove Ferris
        if self.fishes.len() > 1 {
            self.fishes.pop();
        }
    }

    fn ferris(&self) -> Fish {
        return Fish::new(
            Fish::DEFAULT_SPRITE_WIDTH,
            vec2(12., 4.),
            Rect { x: 35., y: 48.5, w: 30., h: 13. },
            Movement::Crab,
            self.ferris_texture,
            self.bubble_texture,
            );
    }

    fn fish(&self) -> Fish {
        return Fish::new(
            Fish::DEFAULT_SPRITE_WIDTH * rand::gen_range(0.6, 1.4),
            vec2(rand::gen_range(8., 14.), rand::gen_range(2.5, 4.5)),
            self.bounding_box,
            Movement::random(),
            *self.fish_textures.choose().unwrap(),
            self.bubble_texture,
            );
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
    switching_backgrounds: bool,
}

impl ShowBackground {
    const BACKGROUND_CHANGE_TIME: f32 = 60.;

    fn new(backgrounds: Vec<Texture2D>) -> Self {
        Self {
            background: 0,
            backgrounds: backgrounds,
            time: 0.,
            switching_backgrounds: true,
        }
    }

    fn draw(&mut self, delta: f32, w: f32, h: f32) {
        if self.time > Self::BACKGROUND_CHANGE_TIME && self.switching_backgrounds {
            self.next();
        }

        self.time += delta;
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

#[macroquad::main(window_conf())]
async fn main() {
    const SCR_W: f32 = 100.0;
    const SCR_H: f32 = 62.5;

    let backgrounds = vec![
        load_texture("assets/background.png").await,
        load_texture("assets/background2.png").await,
        load_texture("assets/background3.png").await,
        load_texture("assets/background4.png").await,
        load_texture("assets/background5.png").await,
        load_texture("assets/background6.png").await,
        load_texture("assets/background7.png").await,
    ];
    let fish_textures = vec![
        load_texture(Fish::SPRITE_CLOWNFISH).await,
        load_texture(Fish::SPRITE_ANGELFISH).await,
        load_texture(Fish::SPRITE_GOLDFISH).await,
        load_texture(Fish::SPRITE_YELLOWFISH).await,
        load_texture(Fish::SPRITE_SEAHORSE).await,
        load_texture(Fish::SPRITE_ROYALGRAMMA).await,
        load_texture(Fish::SPRITE_BUTTERFLYFISH).await,
        load_texture(Fish::SPRITE_LIONFISH).await,
        load_texture(Fish::SPRITE_TURTLE).await,
        load_texture(Fish::SPRITE_NEONTETRA).await,
        load_texture(Fish::SPRITE_YELLOWANGELFISH).await,
        load_texture(Fish::SPRITE_ZEBRAFISH).await,
    ];
    let ferris_texture: Texture2D = load_texture(Fish::SPRITE_CRAB).await;
    let bubble_texture: Texture2D = load_texture(Fish::SPRITE_BUBBLE).await;
    //let submarine: Texture2D = load_texture(Fish::SPRITE_YELLOWSUBMARINE).await;

    let crt_render_target = render_target(screen_width() as u32, screen_height() as u32);
    set_texture_filter(crt_render_target.texture, FilterMode::Linear);
    let water_render_target = render_target(screen_width() as u32, screen_height() as u32);
    set_texture_filter(water_render_target.texture, FilterMode::Linear);
    let water_material = load_material(water_wave_shader::VERTEX, water_wave_shader::FRAGMENT, Default::default()).unwrap();
    let crt_material = load_material(crt_shader::VERTEX, crt_shader::FRAGMENT, Default::default()).unwrap();
    let mut shader_activated = false;
    let mut fish_tank = FishTank::new(SCR_W, SCR_H, ferris_texture, bubble_texture, fish_textures);
    let mut background = ShowBackground::new(backgrounds);
    let mut show_text: ShowText = ShowText::empty();

    fish_tank.populate(10);

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
