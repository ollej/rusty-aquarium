#![windows_subsystem = "windows"]

mod shaders;
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    color::colors::{DARKBLUE, WHITE},
    input::{is_key_pressed, is_mouse_button_pressed, KeyCode, MouseButton},
    material::{gl_use_default_material, gl_use_material},
    math::vec2,
    texture::{draw_texture_ex, render_target, DrawTextureParams, FilterMode},
    time::get_frame_time,
    window::{clear_background, next_frame, screen_height, screen_width, Conf},
};

use rusty_aquarium::{
    config::Config, fish_tank::FishTank, resources::Resources, show_help::ShowHelp,
    show_text::ShowText,
};

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
    let water_material = shaders::water_wave::material().unwrap();
    let crt_material = shaders::crt::material().unwrap();
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
            render_target: Some(water_render_target.clone()),
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
                render_target: Some(crt_render_target.clone()),
                ..Default::default()
            });
            clear_background(DARKBLUE);
            gl_use_material(&water_material);

            draw_texture_ex(
                &water_render_target.texture.clone(),
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

            gl_use_material(&crt_material);

            draw_texture_ex(
                &crt_render_target.texture.clone(),
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
                &water_render_target.texture,
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
