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

    // Setup post processing
    let render_target = render_target(200, 125);
    set_texture_filter(render_target.texture, FilterMode::Nearest);
    let material = load_material(CRT_VERTEX_SHADER, CRT_FRAGMENT_SHADER, Default::default()).unwrap();

    loop {
        // build camera with following coordinate system:
        // (0., 0)     .... (SCR_W, 0.)
        // (0., SCR_H) .... (SCR_W, SCR_H)
        set_camera(Camera2D {
            zoom: vec2(1. / SCR_W * 2., -1. / SCR_H * 2.),
            target: vec2(SCR_W / 2., SCR_H / 2.),
            render_target: Some(render_target),
            ..Default::default()
        });

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

        // Draw fishy
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

        // Post processing effect
        //set_default_camera();
        // 0..1, 0..1 camera
        set_camera(Camera2D {
            zoom: vec2(1.0, 1.0),
            target: vec2(1., 1.),
            ..Default::default()
        });
        clear_background(RED);
        gl_use_material(material);
        draw_texture_ex(
            render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(1.0, 1.0)),
                ..Default::default()
            },
            );
        gl_use_default_material();

        next_frame().await
    }
}

const CRT_FRAGMENT_SHADER: &'static str = r#"#version 100
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

    if (crtUV.x < 0.0 || crtUV.x > 1.0 || crtUV.y < 0.0 || crtUV.y > 1.0) {
        res = vec3(0.0, 0.0, 0.0);

    }
    DrawVignette(res, crtUV);
    DrawScanline(res, uv);
    gl_FragColor = vec4(res, 1.0);
}
"#;

const CRT_VERTEX_SHADER: &'static str = "#version 100
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
";
