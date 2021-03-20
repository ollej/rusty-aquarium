use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;

pub struct Fish {
    position: Vec2,
    speed: Vec2,
    size: Vec2,
    max_position: Vec2,
    texture: Texture2D,
}
impl Fish {
    const SPRITE: &'static str = "resources/clownfish.png";
    const MAX_POSITION: Vec2 = Vec2 { x: 5., y: 10. };
    const MIN_POSITION: Vec2 = Vec2 { x: 5., y: 10. };
    const DIRECTION_CHANGE_CHANCE: Vec2 = Vec2 { x: 2.5, y: 5. };
    const SIZE: f32 = 10.;

    fn new(screen_size: Vec2, texture: Texture2D) -> Fish {
        let fish_height = Fish::SIZE / (texture.width() / texture.height());
        let start_position = vec2(
            rand::gen_range(Fish::MIN_POSITION.x, screen_size.x - Fish::MAX_POSITION.x - Fish::SIZE - 1.),
            rand::gen_range(Fish::MIN_POSITION.y, screen_size.y - Fish::MAX_POSITION.y - fish_height - 1.));
        let size = Vec2 { x: Fish::SIZE, y: fish_height };
        Fish {
            position: start_position,
            speed: Vec2 {
                x: 25. * Fish::random_direction() * Fish::random_speed_modifier(),
                y: 7. * Fish::random_speed_modifier()
            },
            size: size,
            max_position: Vec2 {
                x: screen_size.x - Fish::MAX_POSITION.x - size.x,
                y: screen_size.y - Fish::MAX_POSITION.y
            },
            texture: texture,
        }
    }

    fn random_direction() -> f32 {
        return *vec![-1., 1.].choose().unwrap();
    }

    fn random_percent() -> f32 {
        return rand::gen_range(0., 100.);
    }

    fn random_speed_modifier() -> f32 {
        return rand::gen_range(0.5, 1.1);
    }

    fn tick(&mut self, delta: f32) {
        self.update_position(delta);

        // Change X direction
        if self.position.x < Fish::MIN_POSITION.x {
            self.speed.x *= -1.;
            self.position.x = Fish::MIN_POSITION.x;
        }
        if self.position.x > self.max_position.x {
            self.speed.x *= -1.;
            self.position.x = self.max_position.x;
        }
        if Fish::random_percent() < Fish::DIRECTION_CHANGE_CHANCE.x {
            self.speed.x *= -1.;
        }
        // Change Y direction
        if self.position.y < Fish::MIN_POSITION.y {
            self.speed.y *= -1.;
            self.position.y = Fish::MIN_POSITION.y;
        }
        if self.position.y > self.max_position.y {
            self.speed.y *= -1.;
            self.position.y = self.max_position.y;
        }
        if Fish::random_percent() < Fish::DIRECTION_CHANGE_CHANCE.y {
            self.speed.y *= -1.;
        }
    }

    fn update_position(&mut self, delta: f32) {
        //debug!("x: {} y: {} d: {}", self.position.x, self.position.y, delta);
        self.position = Vec2 {
            x: self.position.x + self.speed.x * delta,
            y: self.position.y + self.speed.y * delta
        };
    }

    fn draw(&mut self) {
        draw_texture_ex(
            self.texture,
            self.position.x,
            self.position.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.size),
                flip_x: self.direction(),
                ..Default::default()
            },
            );
    }

    fn direction(&mut self) -> bool {
        return self.speed.x > 0.;
    }
}

#[macroquad::main("RustyAquarium")]
async fn main() {
    const SCR_W: f32 = 100.0;
    const SCR_H: f32 = 62.5;

    let background: Texture2D = load_texture("resources/background.png").await;
    let fish_texture: Texture2D = load_texture(Fish::SPRITE).await;

    let mut first_frame = true;
    let screen_size = Vec2 { x: SCR_W, y: SCR_H };
    let mut fishies = Vec::new();

    for _ in 0..10 {
        fishies.push(Fish::new(screen_size, fish_texture));
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

