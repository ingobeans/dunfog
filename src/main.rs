use macroquad::{miniquad::window::screen_size, prelude::*, rand};
use utils::*;

mod assets;
mod utils;

#[derive(Clone, Copy)]
enum Tile {
    Floor,
    Wall,
}

struct Dungeon {
    tiles: Vec<Vec<Tile>>,
}
impl Dungeon {
    fn generate_dungeon() -> Self {
        let mut tiles =
            vec![vec![Tile::Wall; SCREEN_WIDTH as usize / 8]; SCREEN_HEIGHT as usize / 8];

        let mut area_left = 5 * 5 * 5;
        let mut rooms: Vec<(usize, usize, usize, usize)> = Vec::new();
        loop {
            let w = rand::gen_range(4, 6);
            let h = rand::gen_range(4, 6);
            let mut x = rand::gen_range(0, SCREEN_WIDTH as usize / 8);
            let mut y = rand::gen_range(0, SCREEN_HEIGHT as usize / 8);
            if x + w >= SCREEN_WIDTH as usize / 8 {
                x = SCREEN_WIDTH as usize / 8 - w - 1;
            }
            if y + h >= SCREEN_HEIGHT as usize / 8 {
                y = SCREEN_HEIGHT as usize / 8 - h - 1;
            }
            let area = w * h;
            if area > area_left {
                break;
            }
            area_left -= area;

            rooms.push((x, y, w, h));
        }
        for (x, y, w, h) in rooms.into_iter() {
            for j in x..x + w {
                for k in y..y + h {
                    tiles[k][j] = Tile::Floor
                }
            }
        }
        Self { tiles }
    }
}

#[macroquad::main("dunfog")]
async fn main() {
    rand::srand(miniquad::date::now() as _);
    let assets = assets::Assets::default();
    let dungeon = Dungeon::generate_dungeon();
    let camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);
    loop {
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        set_camera(&camera);
        clear_background(WHITE);

        for (y, row) in dungeon.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let (tile_x, tile_y) = if let Tile::Floor = *tile {
                    (1.0, 2.0)
                } else {
                    (3.0, 1.0)
                };
                assets.tileset.draw_tile(
                    x as f32 * 8.0 - SCREEN_WIDTH / 2.0,
                    y as f32 * 8.0 - SCREEN_HEIGHT / 2.0,
                    tile_x,
                    tile_y,
                    None,
                );
            }
        }
        assets.tileset.draw_tile(0.0, 0.0, 0.0, 0.0, None);

        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &camera.render_target.as_ref().unwrap().texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(
                    SCREEN_WIDTH * scale_factor,
                    SCREEN_HEIGHT * scale_factor,
                )),
                ..Default::default()
            },
        );
        next_frame().await
    }
}
