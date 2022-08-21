use macroquad_particles::ParticleMaterial;

pub mod crt {
    pub const FRAGMENT: &str = r#"#version 100
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

    pub const VERTEX: &str = r#"#version 100
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

pub mod water_wave {
    pub const FRAGMENT: &str = r#"#version 100
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

    pub const VERTEX: &str = r#"#version 100
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

pub mod water_particle {
    use super::*;

    pub fn material() -> ParticleMaterial {
        ParticleMaterial::new(water_particle::VERTEX, water_particle::PARTICLE)
    }

    pub const VERTEX: &str = r#"#version 100
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

    pub const PARTICLE: &str = r#"#version 100
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
