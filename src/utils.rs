use std::{ops::Range, sync::LazyLock};

use macroquad::{
    miniquad::{BlendFactor, BlendState, BlendValue, Equation},
    prelude::*,
};
pub const SCREEN_WIDTH: f32 = 256.0;
pub const SCREEN_HEIGHT: f32 = 144.0;

pub const SCROLL_AMT: f32 = 1.1;
pub const MIN_ZOOM: f32 = 0.001;

pub const TILES_HORIZONTAL: usize = SCREEN_WIDTH as usize / 8;
pub const TILES_VERTICAL: usize = SCREEN_HEIGHT as usize / 8;

pub const ACTION_TIME: f32 = 0.15;
pub const MAX_PLAYER_HP: f32 = 25.0;

pub fn serialize_range(range: &Range<usize>) -> String {
    let min = range.clone().min();
    let max = range.clone().max();
    if let Some(min) = min
        && max.is_some_and(|f| f == min)
    {
        format!("={}", min)
    } else if min.is_none_or(|f| f <= 1) {
        format!("<{}", max.unwrap())
    } else if max.is_none() {
        format!(">{}", min.unwrap())
    } else {
        format!("{}..{}", min.unwrap(), max.unwrap())
    }
}
pub fn create_camera(w: f32, h: f32) -> Camera2D {
    let rt = render_target(w as u32, h as u32);
    rt.texture.set_filter(FilterMode::Nearest);

    Camera2D {
        render_target: Some(rt),
        zoom: Vec2::new(1.0 / w * 2.0, 1.0 / h * 2.0),
        ..Default::default()
    }
}
pub fn get_input_axis() -> Vec2 {
    let mut i = Vec2::ZERO;
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        i.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        i.x += 1.0;
    }
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        i.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        i.y += 1.0;
    }
    i
}
pub static DAMAGE_MATERIAL: LazyLock<Material> = LazyLock::new(|| {
    // to enable transparency!
    let pipeline = PipelineParams {
        alpha_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        color_blend: Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        )),
        ..Default::default()
    };
    let m = load_material(
        ShaderSource::Glsl {
            vertex: DEFAULT_VERTEX_SHADER,
            fragment: DAMAGE_FRAGMENT,
        },
        MaterialParams {
            pipeline_params: pipeline,
            uniforms: vec![UniformDesc::new("color", UniformType::Float4)],
            ..Default::default()
        },
    )
    .unwrap();
    m.set_uniform("color", Color::from_rgba(255, 0, 25, 100));
    m
});

pub const DAMAGE_FRAGMENT: &str = "#version 100
precision lowp float;

varying vec2 uv;

uniform lowp vec4 color;

uniform sampler2D Texture;

void main() {
    vec4 c = texture2D(Texture, uv);
    if (c.a > 0.0) {
        vec4 fa = vec4(color.rgb,1.0);
        gl_FragColor = mix(c,fa,color.a);
    } else {
        gl_FragColor = texture2D(Texture, uv);
    }
}
";

pub const DEFAULT_VERTEX_SHADER: &str = "#version 100
precision lowp float;

attribute vec3 position;
attribute vec2 texcoord;

varying vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
";

#[cfg(test)]
mod tests {
    use crate::utils::serialize_range;

    #[test]
    fn serialize_range_test() {
        assert_eq!(serialize_range(&(1..5)).as_str(), "<4");
        assert_eq!(serialize_range(&(2..7)).as_str(), "2..6");
        assert_eq!(serialize_range(&(1..2)).as_str(), "=1");
    }
}
