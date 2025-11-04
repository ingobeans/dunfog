use macroquad::{miniquad::window::screen_size, prelude::*, rand};
use utils::*;

mod assets;
mod entities;
mod utils;

#[derive(Clone, Copy)]
enum Tile {
    Floor,
    Wall,
    Path,
}

impl Tile {
    fn is_walkable(self) -> bool {
        match self {
            Tile::Floor | Tile::Path => true,
            _ => false,
        }
    }
    fn get_tile(self) -> (f32, f32) {
        #[expect(unreachable_patterns)]
        match self {
            Tile::Floor | Tile::Path => (0.0, 1.0),
            Tile::Path => (1.0, 1.0),
            Tile::Wall => (0.0, 2.0),
        }
    }
}

struct Dungeon {
    tiles: Vec<Tile>,
    player_spawn: (usize, usize),
}
impl Dungeon {
    fn generate_dungeon() -> Self {
        let mut tiles = vec![Tile::Wall; TILES_HORIZONTAL * TILES_VERTICAL];

        let rooms_area = 5 * 5 * 5;
        let mut area_left = rooms_area;
        let mut rooms: Vec<(usize, usize, usize, usize)> = Vec::new();
        let mut player_spawn = None;
        loop {
            let w = rand::gen_range(4, 6);
            let h = rand::gen_range(4, 6);
            let mut x = rand::gen_range(0, TILES_HORIZONTAL);
            let mut y = rand::gen_range(0, TILES_VERTICAL);
            if x + w >= TILES_HORIZONTAL {
                x = TILES_HORIZONTAL - w - 1;
            }
            if y + h >= TILES_VERTICAL {
                y = TILES_VERTICAL - h - 1;
            }
            let area = w * h;
            if area > area_left {
                break;
            }
            if player_spawn.is_none() {
                player_spawn = Some((x, y))
            }
            area_left -= area;

            rooms.push((x, y, w, h));
        }
        let mut positions = Vec::with_capacity(rooms_area - area_left);

        for (x, y, w, h) in rooms.into_iter() {
            for j in x..x + w {
                for k in y..y + h {
                    positions.push((j, k));
                    tiles[j + k * TILES_HORIZONTAL] = Tile::Floor
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
                    } else if current_y < target_y {
                        current_y += 1;
                    } else {
                        current_y -= 1;
                    }
                }
                if positions.contains(&(current_x, current_y)) {
                    break;
                }
                area_left = area_left.saturating_sub(1);
                tiles[current_x + current_y * TILES_HORIZONTAL] = Tile::Path;
            }
            if is_all_rooms_connected(&tiles) {
                break;
            }
        }

        Self {
            tiles,
            player_spawn: player_spawn.unwrap(),
        }
    }
}

#[macroquad::main("dunfog")]
async fn main() {
    let seed = miniquad::date::now() as u64;
    rand::srand(seed);
    println!("dunfog v{} - seed: {seed}", env!("CARGO_PKG_VERSION"));
    let assets = assets::Assets::default();
    let dungeon = Dungeon::generate_dungeon();
    let mut player = entities::Player::default();
    (player.x, player.y) = dungeon.player_spawn;
    let mut camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);
    loop {
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        let (mouse_x, mouse_y) = mouse_position();

        let (mouse_x, mouse_y) = (mouse_x / scale_factor, mouse_y / scale_factor);
        let mouse_delta = mouse_delta_position();
        let scroll = mouse_wheel();

        if is_mouse_button_down(MouseButton::Middle) {
            player.camera_pos.x += mouse_delta.x as f32 * SCREEN_WIDTH / 2. / player.camera_zoom;
            player.camera_pos.y += mouse_delta.y as f32 * SCREEN_HEIGHT / 2. / player.camera_zoom;
        }
        if scroll.1 != 0.0 {
            let amt = if scroll.1 > 0.0 {
                1.0 / SCROLL_AMT
            } else {
                SCROLL_AMT
            };
            // store old mouse position (in world position)
            let old_mouse_world_x =
                mouse_x / player.camera_zoom + player.camera_pos.x - SCREEN_WIDTH / 2.0;
            let old_mouse_world_y =
                mouse_y / player.camera_zoom + player.camera_pos.y - SCREEN_HEIGHT / 2.0;

            // update grid size
            player.camera_zoom /= amt;
            player.camera_zoom = player.camera_zoom.max(MIN_ZOOM);
            // move camera position to zoom towards cursor
            // by comparing old world mouse position
            player.camera_pos.x =
                old_mouse_world_x + SCREEN_WIDTH / 2.0 - mouse_x / player.camera_zoom;
            player.camera_pos.y =
                old_mouse_world_y + SCREEN_HEIGHT / 2.0 - mouse_y / player.camera_zoom
        }

        camera.target = player.camera_pos;
        set_camera(&camera);
        clear_background(BLACK);

        for (i, tile) in dungeon.tiles.iter().enumerate() {
            let y = i / (SCREEN_WIDTH as usize / 8);
            let x = i % (SCREEN_WIDTH as usize / 8);
            let (tile_x, tile_y) = tile.get_tile();
            assets
                .tileset
                .draw_tile(x as f32 * 8.0, y as f32 * 8.0, tile_x, tile_y, None);
        }
        assets
            .tileset
            .draw_tile((player.x * 8) as f32, (player.y * 8) as f32, 0.0, 0.0, None);

        draw_rectangle(
            mouse_x / player.camera_zoom + player.camera_pos.x - SCREEN_WIDTH / 2.0,
            mouse_y / player.camera_zoom + player.camera_pos.y - SCREEN_HEIGHT / 2.0,
            2.0,
            2.0,
            GREEN,
        );

        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &camera.render_target.as_ref().unwrap().texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(
                    SCREEN_WIDTH * scale_factor * player.camera_zoom,
                    SCREEN_HEIGHT * scale_factor * player.camera_zoom,
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
        let mut tiles = vec![Tile::Wall; TILES_HORIZONTAL * TILES_VERTICAL];
        tiles[0] = Tile::Floor;
        tiles[1] = Tile::Floor;
        assert!(is_all_rooms_connected(&tiles))
    }
}
