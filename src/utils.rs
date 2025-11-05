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

pub fn is_all_rooms_connected(tiles: &[Tile]) -> bool {
    // algorithm:
    // 1. counts total floor tiles.
    // 2. finds first floor tile.
    // 3. recursively follows all floor tile neighbours
    //    until end and counts how many total connected tiles there are

    let total = tiles.iter().filter(|f| f.is_walkable()).count();

    fn count_connected(
        x: usize,
        y: usize,
        tiles: &[Tile],
        marked: &mut Vec<(usize, usize)>,
    ) -> usize {
        if !tiles[x + y * TILES_HORIZONTAL].is_walkable() {
            return 0;
        }
        if marked.contains(&(x, y)) {
            return 0;
        }
        marked.push((x, y));
        let mut count = 1;
        if x > 0 {
            count += count_connected(x - 1, y, tiles, marked);
        }
        if x < TILES_HORIZONTAL - 1 {
            count += count_connected(x + 1, y, tiles, marked);
        }
        if y > 0 {
            count += count_connected(x, y - 1, tiles, marked);
        }
        if y < TILES_VERTICAL - 1 {
            count += count_connected(x, y + 1, tiles, marked);
        }
        count
    }
    // find first floor tile
    for (i, tile) in tiles.iter().enumerate() {
        if tile.is_walkable() {
            let y = i / (TILES_HORIZONTAL);
            let x = i % (TILES_HORIZONTAL);
            return count_connected(x, y, tiles, &mut Vec::new()) == total;
        }
    }
    false
}
