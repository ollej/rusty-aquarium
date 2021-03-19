use macroquad::prelude::*;

pub struct Fish {
    id: u64,
    screen_size: Vec2,
    position: Vec2,
    speed: Vec2,
    size: Vec2,
    direction_change_chance: Vec2,
    max_position: Vec2,
    min_position: Vec2,
    texture: Texture2D,
}
impl Fish {
    const SPRITE: &'static str = "resources/clownfish.png";

    fn new(id: u64, start_position: Vec2, screen_size: Vec2, texture: Texture2D) -> Fish {
        Fish {
            id,
            screen_size,
            position: start_position,
            speed: Vec2 { x: 25., y: 7. },
            size: Vec2 { x: 10., y: 10. / (texture.width() / texture.height()) },
            direction_change_chance: Vec2 { x: 2., y: 0.5 },
            max_position: Vec2 { x: 5., y: 10. },
            min_position: Vec2 { x: 5., y: 10. },
            texture: texture,
        }
    }

    fn tick(&mut self, delta: f32) {
        // Change X direction
        if self.position.x < self.min_position.x || self.position.x > (self.screen_size.x - self.max_position.x - self.size.x) || rand::gen_range(0., 100.) < self.direction_change_chance.x {
            self.speed.x *= -1.;
        }
        // Change Y direction
        if self.position.y < self.min_position.y || self.position.y > (self.screen_size.y - self.max_position.y) || rand::gen_range(0., 100.) < self.direction_change_chance.y {
            self.speed.y *= -1.;
        }

        self.update_position(delta);
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

    let screen_size = Vec2 { x: SCR_W, y: SCR_H };
    let mut fish = Fish::new(1, Vec2 { x: SCR_W / 2., y: SCR_H / 2. }, screen_size, fish_texture);
    let mut first_frame = true;

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

        fish.tick(delta);

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
        fish.draw();

        next_frame().await
    }
}

