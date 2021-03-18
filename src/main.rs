use macroquad::prelude::*;

#[macroquad::main("RustyAquarium")]
async fn main() {
    const SCR_W: f32 = 100.0;
    const SCR_H: f32 = 62.5;

    let mut dx = 25.;
    let mut dy = 7.;

    let background: Texture2D = load_texture("resources/background.png").await;

    let mut fish_x = SCR_W / 2.;
    let mut fish_y = SCR_H / 2.;
    let fish_texture: Texture2D = load_texture("resources/clownfish.png").await;
    let fish_ratio = fish_texture.width() / fish_texture.height();
    let fish_width = 10.;
    let fish_height = 10. / fish_ratio;

    let direction_change_chance = 1.5;
    let max_x = SCR_W - fish_width / 2.;
    let min_x = fish_width / 2.;
    let max_y = SCR_H - fish_height * 1.5;
    let min_y = fish_height;

    // build camera with following coordinate system:
    // (0., 0)     .... (SCR_W, 0.)
    // (0., SCR_H) .... (SCR_W, SCR_H)
    set_camera(Camera2D {
        zoom: vec2(1. / SCR_W * 2., -1. / SCR_H * 2.),
        target: vec2(SCR_W / 2., SCR_H / 2.),
        ..Default::default()
    });

    loop {
        clear_background(DARKBLUE);

        let delta = get_frame_time();

        // Move fish
        fish_x += dx * delta;
        fish_y += dy * delta;

        // Change X direction
        if fish_x < min_x || fish_x > (max_x - fish_width) || rand::gen_range(0., 100.) < direction_change_chance {
            dx *= -1.;
        }
        // Change Y direction
        if fish_y < min_y || fish_y > (max_y - fish_height) || rand::gen_range(0., 100.) < direction_change_chance {
            dy *= -1.;
        }

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

        draw_texture_ex(
            fish_texture,
            fish_x,
            fish_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(fish_width, fish_height)),
                flip_x: dx > 0.,
                ..Default::default()
            },
            );

        next_frame().await
    }
}
