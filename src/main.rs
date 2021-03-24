use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;

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
        if self.position.x < bounding_box.x || self.position.x > bounding_box.right() {
            self.speed.x *= -1.;
        }
        if self.position.y < bounding_box.y || self.position.y > bounding_box.bottom() {
            self.speed.y *= -1.;
        }
    }

    fn change_direction_randomly(&mut self, change_chance: Vec2) {
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
            Movement::Crab => Movement::tick_crab(motion, bounding_box),
            Movement::Random => Movement::tick_random(motion, bounding_box),
        }
    }

    fn random() -> Movement {
        return *vec![Movement::SingleSpeed, Movement::Accelerating, Movement::EdgeIdling, Movement::Random].choose().unwrap();
    }

    fn tick_single_speed(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_direction_randomly(Movement::DIRECTION_CHANGE_CHANCE);
        motion.clamp(bounding_box);
        motion.rotate();
        return motion;
    }

    fn tick_accelerating(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.accelerate();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_direction_randomly(Movement::DIRECTION_CHANGE_CHANCE);
        motion.clamp(bounding_box);
        motion.rotate();
        return motion;
    }

    fn tick_crab(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.accelerate();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_direction_randomly(Movement::DIRECTION_CHANGE_CHANCE * 5.);
        motion.clamp(bounding_box);
        return motion;
    }

    fn tick_random(mut motion: Motion, bounding_box: Rect) -> Motion {
        motion.accelerate();
        motion.random_idling();
        motion.change_direction_by_bounding_box(bounding_box);
        motion.change_direction_randomly(Movement::DIRECTION_CHANGE_CHANCE);
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
    const SPRITE_CRAB: &'static str = "assets/ferris.png";
    const MAX_POSITION: Vec2 = Vec2 { x: 5., y: 5. };
    const MIN_POSITION: Vec2 = Vec2 { x: 5., y: 5. };
    const DEFAULT_SPRITE_WIDTH: f32 = 7.;

    fn new(
        fish_size: f32,
        max_speed: Vec2,
        bounding_box: Rect,
        movement: Movement,
        texture: Texture2D) -> Fish {
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

    fn swims_right(&mut self) -> bool {
        return self.motion.speed.x >= 0.;
    }

    fn draw(&mut self) {
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

#[macroquad::main("RustyAquarium")]
async fn main() {
    const SCR_W: f32 = 100.0;
    const SCR_H: f32 = 62.5;

    let background: Texture2D = load_texture("assets/background.png").await;
    let ferris: Texture2D = load_texture(Fish::SPRITE_CRAB).await;
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
    ];

    let mut first_frame = true;
    let mut fishies = Vec::new();
    let bounding_box = Rect {
        x: Fish::MIN_POSITION.x,
        y: Fish::MIN_POSITION.y,
        w: SCR_W - Fish::MAX_POSITION.x - Fish::MIN_POSITION.x,
        h: SCR_H - Fish::MAX_POSITION.y - Fish::MIN_POSITION.y,
    };

    let crab_box = Rect { x: 35., y: 48.5, w: 30., h: 13. };
    fishies.push(Fish::new(Fish::DEFAULT_SPRITE_WIDTH, vec2(12., 4.), crab_box, Movement::Crab, ferris));
    for _ in 0..20 {
        let texture = fish_textures.choose().unwrap();
        let size = Fish::DEFAULT_SPRITE_WIDTH * rand::gen_range(0.6, 1.4);
        let max_speed = vec2(rand::gen_range(8., 14.), rand::gen_range(2.5, 4.5));
        fishies.push(Fish::new(size, max_speed, bounding_box, Movement::random(), *texture));
    }

    // build camera with following coordinate system:
    // (0., 0)     .... (SCR_W, 0.)
    // (0., SCR_H) .... (SCR_W, SCR_H)
    set_camera(Camera2D {
        zoom: vec2(1. / SCR_W * 2., -1. / SCR_H * 2.),
        target: vec2(SCR_W / 2., SCR_H / 2.),
        ..Default::default()
    });

    loop {
        // Skip the first frame as the delta is too high
        if first_frame {
            first_frame = false;
            next_frame().await;
            continue;
        }

        clear_background(DARKBLUE);

        let delta = get_frame_time();

        for fish in fishies.iter_mut() {
            fish.tick(delta);
        }

        // Draw background
        draw_texture_ex(
            background,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(SCR_W, SCR_H)),
                ..Default::default()
            },
            );
        for fish in fishies.iter_mut() {
            fish.draw();
        }

        next_frame().await
    }
}

