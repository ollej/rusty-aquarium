use macroquad::prelude::*;

#[macroquad::main("RustyAquarium")]
async fn main() {
    const SCR_W: f32 = 100.0;
    const SCR_H: f32 = 62.5;

    let mut ball_x = SCR_W / 2.;
    let mut ball_y = SCR_H / 2.;
    let mut dx = 30.;
    let mut dy = 30.;
    let ball_size = 1.;

    // build camera with following coordinate system:
    // (0., 0)     .... (SCR_W, 0.)
    // (0., SCR_H) .... (SCR_W, SCR_H)
    set_camera(Camera2D {
        zoom: vec2(1. / SCR_W * 2., -1. / SCR_H * 2.),
        target: vec2(SCR_W / 2., SCR_H / 2.),
        ..Default::default()
    });

    loop {
        clear_background(PURPLE);

        let delta = get_frame_time();

        // Move ball
        ball_x += dx * delta;
        ball_y += dy * delta;

        // Change X direction
        if ball_x <= ball_size / 2. || ball_x > (SCR_W - ball_size / 2.) {
            dx *= -1.;
        }
        // Change Y direction
        if ball_y <= ball_size / 2. || ball_y > (SCR_H - ball_size / 2.) {
            dy *= -1.;
        }

        draw_circle(ball_x, ball_y, ball_size, GREEN);

        next_frame().await
    }
}
