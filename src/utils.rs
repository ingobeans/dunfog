use macroquad::prelude::*;

use crate::Tile;

pub const SCREEN_WIDTH: f32 = 256.0;
pub const SCREEN_HEIGHT: f32 = 144.0;

pub const SCROLL_AMT: f32 = 1.1;
pub const MIN_ZOOM: f32 = 0.001;

pub const TILES_HORIZONTAL: usize = SCREEN_WIDTH as usize / 8;
pub const TILES_VERTICAL: usize = SCREEN_HEIGHT as usize / 8;

pub const ACTION_TIME: f32 = 0.15;

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
