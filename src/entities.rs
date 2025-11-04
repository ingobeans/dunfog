use crate::utils::*;
use macroquad::prelude::*;

pub struct Player {
    pub x: usize,
    pub y: usize,
    pub camera_pos: Vec2,
    pub camera_zoom: f32,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            camera_pos: vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0),
            camera_zoom: 1.0,
        }
    }
}
