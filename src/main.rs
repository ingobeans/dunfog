use macroquad::{miniquad::window::screen_size, prelude::*, rand};
use utils::*;

use crate::assets::Assets;

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
        matches!(self, Tile::Floor | Tile::Path)
    }
    fn get_tile(self) -> (f32, f32) {
        #[expect(unreachable_patterns)]
        match self {
            Tile::Floor | Tile::Path => (0.0, 1.0),
            Tile::Path => (1.0, 1.0),
            Tile::Wall => (0.0, 0.0),
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
                } else if current_y == target_y {
                    moving_horizontal = true;
                } else if current_y < target_y {
                    current_y += 1;
                } else {
                    current_y -= 1;
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

struct Dunfog<'a> {
    player: entities::Player,
    dungeon: Dungeon,
    assets: &'a Assets,
    camera: Camera2D,
}
impl<'a> Dunfog<'a> {
    fn new(assets: &'a Assets) -> Self {
        let dungeon = Dungeon::generate_dungeon();
        let mut player = entities::Player::default();
        (player.x, player.y) = dungeon.player_spawn;
        let mut camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);
        camera.target = vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);
        Self {
            player,
            dungeon,
            assets,
            camera,
        }
    }
    fn update(&mut self) {
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        let (mouse_x, mouse_y) = mouse_position();

        let (mouse_x, mouse_y) = (mouse_x / scale_factor, mouse_y / scale_factor);
        let (mouse_tile_x, mouse_tile_y) = (
            (((mouse_x) / self.player.camera_zoom + self.player.camera_pos.x) / 8.0).floor(),
            (((mouse_y) / self.player.camera_zoom + self.player.camera_pos.y) / 8.0).floor(),
        );

        let mouse_delta = mouse_delta_position();
        let scroll = mouse_wheel();

        if is_mouse_button_down(MouseButton::Middle) {
            self.player.camera_pos.x +=
                mouse_delta.x as f32 * SCREEN_WIDTH / 2. / self.player.camera_zoom;
            self.player.camera_pos.y +=
                mouse_delta.y as f32 * SCREEN_HEIGHT / 2. / self.player.camera_zoom;
        }
        if scroll.1 != 0.0 {
            let amt = if scroll.1 > 0.0 {
                1.0 / SCROLL_AMT
            } else {
                SCROLL_AMT
            };
            // store old mouse position (in world position)
            let old_mouse_world_x =
                mouse_x / self.player.camera_zoom + self.player.camera_pos.x - SCREEN_WIDTH / 2.0;
            let old_mouse_world_y =
                mouse_y / self.player.camera_zoom + self.player.camera_pos.y - SCREEN_HEIGHT / 2.0;

            // update grid size
            self.player.camera_zoom /= amt;
            self.player.camera_zoom = self.player.camera_zoom.max(MIN_ZOOM);
            // move camera position to zoom towards cursor
            // by comparing old world mouse position
            self.player.camera_pos.x =
                old_mouse_world_x + SCREEN_WIDTH / 2.0 - mouse_x / self.player.camera_zoom;
            self.player.camera_pos.y =
                old_mouse_world_y + SCREEN_HEIGHT / 2.0 - mouse_y / self.player.camera_zoom
        }

        set_camera(&self.camera);
        clear_background(BLACK);

        for (i, tile) in self.dungeon.tiles.iter().enumerate() {
            let y = i / (SCREEN_WIDTH as usize / 8);
            let x = i % (SCREEN_WIDTH as usize / 8);
            let (tile_x, tile_y) = tile.get_tile();
            self.assets
                .tileset
                .draw_tile(x as f32 * 8.0, y as f32 * 8.0, tile_x, tile_y, None);
        }
        self.assets.tileset.draw_tile(
            (self.player.x * 8) as f32,
            (self.player.y * 8) as f32,
            1.0,
            0.0,
            None,
        );

        if mouse_tile_x >= 0.0
            && mouse_tile_y >= 0.0
            && mouse_tile_x < TILES_HORIZONTAL as f32
            && mouse_tile_y < TILES_VERTICAL as f32
        {
            let (tile_x, tile_y) = (mouse_tile_x as usize, mouse_tile_y as usize);
            if self.dungeon.tiles[tile_x + tile_y * TILES_HORIZONTAL].is_walkable() {
                self.assets.tileset.draw_tile(
                    mouse_tile_x * 8.0,
                    mouse_tile_y * 8.0,
                    2.0,
                    0.0,
                    None,
                );
            }
        }

        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &self.camera.render_target.as_ref().unwrap().texture,
            -self.player.camera_pos.x * scale_factor * self.player.camera_zoom,
            -self.player.camera_pos.y * scale_factor * self.player.camera_zoom,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(
                    SCREEN_WIDTH * scale_factor * self.player.camera_zoom,
                    SCREEN_HEIGHT * scale_factor * self.player.camera_zoom,
                )),
                ..Default::default()
            },
        );
    }
}

#[macroquad::main("dunfog")]
async fn main() {
    let seed = miniquad::date::now().to_bits();
    rand::srand(seed);
    println!("dunfog v{} - seed: {seed}", env!("CARGO_PKG_VERSION"));
    let assets = assets::Assets::default();
    let mut dunfog = Dunfog::new(&assets);
    loop {
        dunfog.update();
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
