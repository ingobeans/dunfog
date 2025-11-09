use macroquad::{miniquad::window::screen_size, prelude::*, rand};
use utils::*;

use crate::{
    assets::Assets, dungeon::*, entities::*, items::StatusEffect, loot::LootTable,
    ui::InventoryState,
};

mod assets;
mod dungeon;
mod entities;
mod items;
mod loot;
mod particles;
mod ui;
mod utils;

#[derive(Clone, Copy)]
enum Tile {
    Floor,
    Wall,
    Path,
    Door,
    Chest(f32, f32, &'static LootTable),
    Detail(f32, f32),
}

impl Tile {
    fn is_walkable(self) -> bool {
        !matches!(self, Tile::Wall)
    }
}

#[expect(dead_code)]
enum GameState {
    Idle,
    PlayerAction(f32),
    Waiting(f32),
    EnemyAction(f32),
}
impl GameState {
    fn get_time(&self) -> f32 {
        match self {
            GameState::Idle => 0.0,
            GameState::PlayerAction(time) => (ACTION_TIME - *time) / ACTION_TIME,
            GameState::Waiting(time) => (ACTION_TIME - *time) / ACTION_TIME,
            GameState::EnemyAction(time) => (ACTION_TIME - *time) / ACTION_TIME,
        }
    }
}

struct Dunfog<'a> {
    player: Player,
    floor: usize,
    dungeon: Dungeon,
    assets: &'a Assets,
    world_camera: Camera2D,
    state: GameState,
    inv_state: InventoryState,
    dead: Option<f32>,
}
impl<'a> Dunfog<'a> {
    fn new(assets: &'a Assets, dungeon: Dungeon) -> Self {
        let mut player = Player::default();
        player.move_to(dungeon.player_spawn, &dungeon);
        player.center_camera((SCREEN_WIDTH, SCREEN_HEIGHT));
        let mut world_camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);
        world_camera.target = vec2(SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0);
        Self {
            player,
            dungeon,
            floor: 0,
            assets,
            world_camera,
            state: GameState::Idle,
            inv_state: InventoryState::Closed,
            dead: None,
        }
    }
    fn die(&mut self) {
        self.dead = Some(0.0);
        self.inv_state = InventoryState::Closed;
        self.state = GameState::Idle;
    }
    fn perform_enemy_actions(&mut self) {
        let mut buffer = Vec::new();
        // me when i can just use mem::swap to get around a fundemental problem with my memory structure
        std::mem::swap(&mut buffer, &mut self.dungeon.enemies);
        let mut enemy_positions: Vec<(usize, usize)> = buffer.iter().map(|f| (f.x, f.y)).collect();
        for enemy in buffer.iter_mut() {
            if enemy.awake {
                let action = enemy.act(&mut self.dungeon, &mut self.player);
                if let EnemyAction::MoveTo(pos) = action {
                    if (self.player.x, self.player.y) == pos || enemy_positions.contains(&pos) {
                        enemy.current_action = Some(EnemyAction::Wait)
                    } else {
                        *enemy_positions
                            .iter_mut()
                            .find(|f| **f == (enemy.x, enemy.y))
                            .expect("enemy not found at its own location?") = pos;
                        enemy.current_action = Some(action);
                        (enemy.x, enemy.y) = pos;
                    }
                } else {
                    enemy.current_action = Some(action)
                }
            } else if matches!(
                self.player.tile_status[enemy.x + enemy.y * TILES_HORIZONTAL],
                TileStatus::Known
            ) && rand::gen_range(
                0,
                (enemy.x.abs_diff(self.player.x) + enemy.y.abs_diff(self.player.y)).min(14),
            ) <= 2
            {
                enemy.awaken();
            }
        }
        std::mem::swap(&mut buffer, &mut self.dungeon.enemies);
    }
    fn update_gamestate(&mut self, delta_time: f32) {
        match &mut self.state {
            GameState::Idle => {}
            GameState::PlayerAction(t) => {
                *t -= delta_time;
                if *t <= 0.0 {
                    let enemies_visible =
                        !self.player.get_visible_enemies(&self.dungeon).is_empty();
                    self.dungeon.particles.clear();

                    for (k, v) in self.player.status_effects.iter_mut() {
                        if let StatusEffect::Poison = k {
                            self.player.health -= 2.0;
                            self.player.was_damaged = true;
                        }

                        *v -= 1;
                    }
                    self.player.status_effects.retain(|_, v| *v > 0);

                    self.perform_enemy_actions();
                    if enemies_visible {
                        self.state = GameState::EnemyAction(ACTION_TIME);
                    } else {
                        self.state = GameState::Idle;
                        self.dungeon.particles.clear();
                    }
                }
            }
            GameState::Waiting(t) => {
                *t -= delta_time;
                if *t <= 0.0 {
                    self.state = GameState::EnemyAction(ACTION_TIME);
                }
            }
            GameState::EnemyAction(t) => {
                *t -= delta_time;
                if *t <= 0.0 {
                    self.state = GameState::Idle;
                    self.dungeon.particles.clear();
                }
            }
        }
    }
    fn update(&mut self) {
        if self.player.health <= 0.0 && self.dead.is_none() {
            self.die();
        }
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        let (mouse_x, mouse_y) = mouse_position();

        let delta_time = get_frame_time();

        if (is_key_pressed(KeyCode::F) || is_key_pressed(KeyCode::Escape))
            && self.player.should_throw_item.is_none()
        {
            self.inv_state.toggle();
        }

        if let Some(index) = self.player.should_drop_item {
            self.player.should_drop_item = None;
            let item = self.player.inventory[index].take().unwrap();
            self.dungeon
                .items
                .push((self.player.x, self.player.y, item));
        }

        let (mouse_x, mouse_y) = (mouse_x / scale_factor, mouse_y / scale_factor);
        let (mouse_tile_x, mouse_tile_y) = (
            (((mouse_x) / self.player.camera_zoom + self.player.camera_pos.x) / 8.0).floor(),
            (((mouse_y) / self.player.camera_zoom + self.player.camera_pos.y) / 8.0).floor(),
        );

        let mouse_delta = mouse_delta_position();
        let scroll = mouse_wheel();

        if is_mouse_button_down(MouseButton::Middle) && self.dead.is_none() {
            self.player.camera_pos.x += mouse_delta.x as f32 * actual_screen_width
                / scale_factor
                / 2.
                / self.player.camera_zoom;
            self.player.camera_pos.y += mouse_delta.y as f32 * actual_screen_height
                / scale_factor
                / 2.
                / self.player.camera_zoom;
        }
        if scroll.1 != 0.0 && self.dead.is_none() {
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

        if let GameState::PlayerAction(_) = &self.state
            && let Some(PlayerAction::MoveDirection(dir)) = &self.player.active_action
            && !is_mouse_button_down(MouseButton::Middle)
        {
            let max_dist = 16.0;
            let pos = self.player.draw_pos;
            let screen = vec2(
                actual_screen_width / scale_factor,
                actual_screen_height / scale_factor,
            );
            let camera_world = self.player.camera_pos + screen / 2.0 / self.player.camera_zoom;
            let delta = camera_world - pos;
            if delta.x > max_dist && dir.x < 0.0 {
                self.player.camera_pos.x =
                    max_dist + pos.x - screen.x / 2.0 / self.player.camera_zoom;
            }
            if delta.x < -max_dist && dir.x > 0.0 {
                self.player.camera_pos.x =
                    -max_dist + pos.x - screen.x / 2.0 / self.player.camera_zoom;
            }
            if delta.y > max_dist && dir.y < 0.0 {
                self.player.camera_pos.y =
                    max_dist + pos.y - screen.y / 2.0 / self.player.camera_zoom;
            }
            if delta.y < -max_dist && dir.y > 0.0 {
                self.player.camera_pos.y =
                    -max_dist + pos.y - screen.y / 2.0 / self.player.camera_zoom;
            }
        }

        let mut click = None;
        if let Some(cursor_tile) = cursor_tile
            && is_mouse_button_pressed(MouseButton::Left)
        {
            click = Some(cursor_tile)
        }

        if matches!(self.inv_state, InventoryState::Closed) && self.dead.is_none() {
            self.update_gamestate(delta_time);
            if let Some(action) =
                self.player
                    .update(&mut self.dungeon, delta_time, &self.state, click)
            {
                if let PlayerAction::GotoNextDungeon = action {
                    self.floor += 1;
                    self.dungeon = Dungeon::generate_dungeon(&DUNGEON_FLOORS[self.floor]);
                    self.player.tile_status =
                        vec![TileStatus::Unknown; TILES_HORIZONTAL * TILES_VERTICAL];
                    self.player
                        .move_to(self.dungeon.player_spawn, &self.dungeon);
                    self.player.center_camera((
                        actual_screen_width / scale_factor,
                        actual_screen_height / scale_factor,
                    ));
                } else {
                    self.player.active_action = Some(action);
                    self.state = GameState::PlayerAction(ACTION_TIME);
                }
            }
            for enemy in self.dungeon.enemies.iter_mut() {
                enemy.update(delta_time, &self.state);
            }
        }

        set_camera(&self.world_camera);
        clear_background(BLACK);

        for (i, tile) in self.dungeon.tiles.iter().enumerate() {
            let y = i / TILES_HORIZONTAL;
            let x = i % TILES_HORIZONTAL;
            let (tile_x, tile_y) = (self.dungeon.dungeon_floor.get_sprite)(tile);
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
                    TileStatus::Unknown => draw_texture(
                        &self.assets.darkness,
                        x as f32 * 8.0 - 3.0,
                        y as f32 * 8.0 - 3.0,
                        WHITE,
                    ),
                    TileStatus::Remembered => draw_texture(
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
        let dead = self.dungeon.enemies.extract_if(.., |f| f.health <= 0.0);
        for enemy in dead {
            self.player.enemies_slayed += 1;
            if let Some(loot_table) = enemy.ty.death_drops
                && let Some(item) = loot_table.get_item()
            {
                self.dungeon.items.push((enemy.x, enemy.y, *item));
            }
        }

        for enemy in self.dungeon.enemies.iter() {
            if let TileStatus::Known = self.player.tile_status[enemy.x + enemy.y * TILES_HORIZONTAL]
            {
                enemy.draw(self.assets, time);
            }
        }
        for (x, y, item) in self.dungeon.items.iter() {
            if let TileStatus::Known = self.player.tile_status[x + y * TILES_HORIZONTAL] {
                let sprite = item.get_sprite();
                self.assets.items.draw_tile(
                    (x * 8) as f32,
                    (y * 8) as f32,
                    sprite.x,
                    sprite.y,
                    None,
                );
            }
        }
        self.player.draw(self.assets, time);
        for particle in self.dungeon.particles.iter_mut() {
            particle.draw(self.state.get_time(), self.assets);
        }

        if !matches!(self.inv_state, InventoryState::Inventory(_))
            && let Some((tile_x, tile_y)) = cursor_tile
            && self.dungeon.tiles[tile_x + tile_y * TILES_HORIZONTAL].is_walkable()
            && !self.player.tile_status[tile_x + tile_y * TILES_HORIZONTAL].is_unknown()
        {
            self.assets
                .tileset
                .draw_tile(mouse_tile_x * 8.0, mouse_tile_y * 8.0, 2.0, 0.0, None);
        }
        set_default_camera();
        clear_background(BLACK);

        draw_texture_ex(
            &self.world_camera.render_target.as_ref().unwrap().texture,
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

        ui::draw_ui(
            &mut self.inv_state,
            &mut self.player,
            self.assets,
            &self.dungeon,
        );
        if let Some(dead_time) = &mut self.dead {
            *dead_time += delta_time;
            if ui::draw_dead_screen(*dead_time, self.assets, &self.player, self.floor) {
                *self = Self::new(self.assets, Dungeon::generate_dungeon(&DUNGEON_FLOORS[0]))
            }
        }
    }
}

#[macroquad::main("dunfog")]
async fn main() {
    let use_testing_dungeon = std::env::args().any(|f| f.as_str() == "test");
    let seed = miniquad::date::now().to_bits();
    rand::srand(seed);
    println!("dunfog v{} - seed: {seed}", env!("CARGO_PKG_VERSION"));
    let assets = assets::Assets::default();

    let dungeon = if use_testing_dungeon {
        Dungeon::load_from_file(
            Image::from_file_with_format(include_bytes!("../assets/testing_map.png"), None)
                .unwrap(),
        )
    } else {
        Dungeon::generate_dungeon(&DUNGEON_FLOORS[0])
    };

    let mut dunfog = Dunfog::new(&assets, dungeon);
    loop {
        dunfog.update();
        next_frame().await
    }
}
