use macroquad::{miniquad::window::screen_size, prelude::*, rand};
use utils::*;

mod assets;
mod utils;

#[derive(Clone, Copy)]
enum Tile {
    Floor,
    Wall,
}

fn is_all_rooms_connected(tiles: &[Tile]) -> bool {
    // algorithm:
    // 1. counts total floor tiles.
    // 2. finds first floor tile.
    // 3. recursively follows all floor tile neighbours
    //    until end and counts how many total connected tiles there are

    let total = tiles.iter().filter(|f| matches!(**f, Tile::Floor)).count();

    fn count_connected(
        x: usize,
        y: usize,
        tiles: &[Tile],
        marked: &mut Vec<(usize, usize)>,
    ) -> usize {
        if !matches!(tiles[x + y * SCREEN_WIDTH as usize / 8], Tile::Floor) {
            return 0;
        }
        if marked.contains(&(x, y)) {
            return 0;
        }
        marked.push((x, y));
        let mut count = 1;
        if x > 0 {
            count += count_connected(x - 1, y, tiles, marked);
            if y > 0 {
                count += count_connected(x - 1, y - 1, tiles, marked);
            }
            if y < SCREEN_HEIGHT as usize / 8 - 1 {
                count += count_connected(x - 1, y + 1, tiles, marked);
            }
        }
        if x < SCREEN_WIDTH as usize / 8 - 1 {
            count += count_connected(x + 1, y, tiles, marked);
            if y > 0 {
                count += count_connected(x + 1, y - 1, tiles, marked);
            }
            if y < SCREEN_HEIGHT as usize / 8 - 1 {
                count += count_connected(x + 1, y + 1, tiles, marked);
            }
        }
        if y > 0 {
            count += count_connected(x, y - 1, tiles, marked);
        }
        if y < SCREEN_HEIGHT as usize / 8 - 1 {
            count += count_connected(x, y + 1, tiles, marked);
        }
        count
    }
    // find first floor tile
    for (i, tile) in tiles.iter().enumerate() {
        if matches!(*tile, Tile::Floor) {
            let y = i / (SCREEN_WIDTH as usize / 8);
            let x = i % (SCREEN_WIDTH as usize / 8);
            return count_connected(x, y, tiles, &mut Vec::new()) == total;
        }
    }
    println!("no floor tiles!");
    false
}

struct Dungeon {
    tiles: Vec<Tile>,
}
impl Dungeon {
    fn generate_dungeon() -> Self {
        let mut tiles = vec![Tile::Wall; SCREEN_WIDTH as usize / 8 * SCREEN_HEIGHT as usize / 8];

        let rooms_area = 5 * 5 * 5;
        let mut area_left = rooms_area;
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
        let mut positions = Vec::with_capacity(rooms_area - area_left);

        for (x, y, w, h) in rooms.into_iter() {
            for j in x..x + w {
                for k in y..y + h {
                    positions.push((j, k));
                    tiles[j + k * SCREEN_WIDTH as usize / 8] = Tile::Floor
                }
            }
        }

        area_left = 15;
        loop {
            let (origin_x, origin_y) = positions[rand::gen_range(0, positions.len())];
            let (target_x, target_y) = positions[rand::gen_range(0, positions.len())];
            let delta_x = origin_x.abs_diff(target_x);
            let delta_y = origin_y.abs_diff(target_y);
            let mut moving_horizontal = delta_x < delta_y;
            let (mut current_x, mut current_y) = (origin_x, origin_y);
            loop {
                if current_x == target_x && current_y == target_y {
                    break;
                }
                if moving_horizontal {
                    if current_x == target_x {
                        moving_horizontal = false;
                    } else if current_x < target_x {
                        current_x += 1;
                    } else {
                        current_x -= 1;
                    }
                } else {
                    if current_y == target_y {
                        moving_horizontal = true;
                    }
                    if current_y < target_y {
                        current_y += 1;
                    } else {
                        current_y -= 1;
                    }
                }
                if positions.contains(&(current_x, current_y)) {
                    break;
                }
                area_left = area_left.saturating_sub(1);
                tiles[current_x + current_y * SCREEN_WIDTH as usize / 8] = Tile::Floor;
            }
            if is_all_rooms_connected(&tiles) {
                break;
            }
        }

        Self { tiles }
    }
}

#[macroquad::main("dunfog")]
async fn main() {
    let seed = miniquad::date::now() as u64;
    rand::srand(seed);
    println!("dunfog v{} - seed: {seed}", env!("CARGO_PKG_VERSION"));
    let assets = assets::Assets::default();
    let dungeon = Dungeon::generate_dungeon();
    let camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);
    loop {
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        set_camera(&camera);
        clear_background(BLACK);

        for (i, tile) in dungeon.tiles.iter().enumerate() {
            let y = i / (SCREEN_WIDTH as usize / 8);
            let x = i % (SCREEN_WIDTH as usize / 8);
            let (tile_x, tile_y) = if let Tile::Floor = *tile {
                (0.0, 1.0)
            } else {
                (2.0, 1.0)
            };
            assets.tileset.draw_tile(
                x as f32 * 8.0 - SCREEN_WIDTH / 2.0,
                y as f32 * 8.0 - SCREEN_HEIGHT / 2.0,
                tile_x,
                tile_y,
                None,
            );
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

#[cfg(test)]
mod tests {
    use crate::{Tile, is_all_rooms_connected, utils::*};

    #[test]
    fn test_rooms_connected() {
        let mut tiles = vec![Tile::Wall; SCREEN_WIDTH as usize / 8 * SCREEN_HEIGHT as usize / 8];
        tiles[0] = Tile::Floor;
        tiles[1] = Tile::Floor;
        assert!(is_all_rooms_connected(&tiles))
    }
}
