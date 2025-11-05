use macroquad::{miniquad::window::screen_size, prelude::*, rand};
use utils::*;

use crate::{assets::Assets, dungeon::Dungeon};

mod assets;
mod dungeon;
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

struct Dunfog<'a> {
    player: entities::Player,
    dungeon: Dungeon,
    assets: &'a Assets,
    camera: Camera2D,
    /// When <= 0.0, no action is currently being performed.
    /// Game is idle, waiting for player to act. When > 0.0,
    /// game is currently showing the animation of the current action
    action_animation_active: f32,
}
impl<'a> Dunfog<'a> {
    fn new(assets: &'a Assets) -> Self {
        let dungeon = Dungeon::generate_dungeon();
        let mut player = entities::Player::default();
        player.move_to(dungeon.player_spawn, &dungeon);
        player.center_camera((SCREEN_WIDTH, SCREEN_HEIGHT));
        let mut camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);
        camera.target = vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);
        Self {
            player,
            dungeon,
            assets,
            camera,
            action_animation_active: 0.0,
        }
    }
    fn update(&mut self) {
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        let (mouse_x, mouse_y) = mouse_position();

        let delta_time = get_frame_time();

        let (mouse_x, mouse_y) = (mouse_x / scale_factor, mouse_y / scale_factor);
        let (mouse_tile_x, mouse_tile_y) = (
            (((mouse_x) / self.player.camera_zoom + self.player.camera_pos.x) / 8.0).floor(),
            (((mouse_y) / self.player.camera_zoom + self.player.camera_pos.y) / 8.0).floor(),
        );

        let mouse_delta = mouse_delta_position();
        let scroll = mouse_wheel();

        if is_mouse_button_down(MouseButton::Middle) {
            self.player.camera_pos.x += mouse_delta.x as f32 * actual_screen_width
                / scale_factor
                / 2.
                / self.player.camera_zoom;
            self.player.camera_pos.y += mouse_delta.y as f32 * actual_screen_height
                / scale_factor
                / 2.
                / self.player.camera_zoom;
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
                old_mouse_world_y + SCREEN_HEIGHT / 2.0 - mouse_y / self.player.camera_zoom;
        }
        let cursor_tile = if mouse_tile_x >= 0.0
            && mouse_tile_y >= 0.0
            && mouse_tile_x < TILES_HORIZONTAL as f32
            && mouse_tile_y < TILES_VERTICAL as f32
        {
            Some((mouse_tile_x as usize, mouse_tile_y as usize))
        } else {
            None
        };
        let mut click = None;
        if let Some(cursor_tile) = cursor_tile
            && is_mouse_button_pressed(MouseButton::Left)
        {
            click = Some(cursor_tile)
        }

        if self.action_animation_active <= 0.0 {
            if self
                .player
                .update_idle(&mut self.dungeon, delta_time, click)
            {
                self.action_animation_active = ACTION_TIME;
            }
        } else {
            self.action_animation_active -= delta_time;
            self.player.update_action(
                &mut self.dungeon,
                delta_time,
                ACTION_TIME - self.action_animation_active,
            );
        }

        set_camera(&self.camera);
        clear_background(BLACK);

        for (i, tile) in self.dungeon.tiles.iter().enumerate() {
            let y = i / TILES_HORIZONTAL;
            let x = i % TILES_HORIZONTAL;
            let (tile_x, tile_y) = tile.get_tile();
            if !self.player.tile_status[x + y * TILES_HORIZONTAL].is_unknown() {
                self.assets
                    .tileset
                    .draw_tile(x as f32 * 8.0, y as f32 * 8.0, tile_x, tile_y, None);
            }
        }
        for (index, tile_status) in self.player.tile_status.iter().enumerate() {
            let x = index % TILES_HORIZONTAL;
            let y = index / TILES_HORIZONTAL;

            if self.dungeon.tiles[index].is_walkable() {
                match tile_status {
                    entities::TileStatus::Unknown => draw_texture(
                        &self.assets.darkness,
                        x as f32 * 8.0 - 3.0,
                        y as f32 * 8.0 - 3.0,
                        WHITE,
                    ),
                    entities::TileStatus::Remembered => draw_texture(
                        &self.assets.semi_darkness,
                        x as f32 * 8.0 - 3.0,
                        y as f32 * 8.0 - 3.0,
                        WHITE,
                    ),
                    _ => {}
                }
            }
        }

        let time = get_time();
        self.dungeon.enemies.retain(|f| f.health > 0.0);
        for enemy in self.dungeon.enemies.iter() {
            if let entities::TileStatus::Known =
                self.player.tile_status[enemy.x + enemy.y * TILES_HORIZONTAL]
            {
                enemy.draw(self.assets, time);
            }
        }
        self.player.draw(self.assets, time);

        if let Some((tile_x, tile_y)) = cursor_tile {
            if self.dungeon.tiles[tile_x + tile_y * TILES_HORIZONTAL].is_walkable()
                && !self.player.tile_status[tile_x + tile_y * TILES_HORIZONTAL].is_unknown()
            {
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
