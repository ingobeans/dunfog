use std::f32::consts::PI;

use crate::{GameState, assets, dungeon::Dungeon, utils::*};
use macroquad::prelude::*;

pub enum PlayerAction {
    MoveDirection(Vec2),
    Attack(Vec2),
}
#[derive(Clone, Copy)]
pub enum TileStatus {
    Unknown,
    Known,
    Remembered,
}
impl TileStatus {
    pub fn is_unknown(self) -> bool {
        matches!(self, TileStatus::Unknown)
    }
}
pub struct Weapon {
    pub attack_range: std::ops::Range<usize>,
    pub base_damage: f32,
}
pub const DAGGER: Weapon = Weapon {
    attack_range: 0..1,
    base_damage: 2.0,
};
#[expect(dead_code)]
pub const BOW: Weapon = Weapon {
    attack_range: 1..3,
    base_damage: 2.0,
};
pub struct Player {
    pub active_action: Option<PlayerAction>,
    pub moving_to: Vec<(usize, usize)>,
    pub x: usize,
    pub y: usize,
    pub draw_pos: Vec2,
    pub camera_pos: Vec2,
    pub camera_zoom: f32,
    pub tile_status: Vec<TileStatus>,
    pub active_weapon: Weapon,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            active_action: None,
            moving_to: Vec::new(),
            x: 0,
            y: 0,
            draw_pos: Vec2::ZERO,
            camera_pos: vec2(0.0, 0.0),
            camera_zoom: 1.0,
            tile_status: vec![TileStatus::Unknown; TILES_HORIZONTAL * TILES_VERTICAL],
            active_weapon: DAGGER,
        }
    }
}
impl Player {
    pub fn draw(&self, assets: &assets::Assets, _time_since_start: f64) {
        assets
            .tileset
            .draw_tile(self.draw_pos.x, self.draw_pos.y, 0.0, 2.0, None);
    }
    /// Uses raycasts to see which tiles should be visible by player
    pub fn get_visible_tiles(&mut self, dungeon: &Dungeon) {
        let directions = 32;
        let steps = 5;

        for tile in self.tile_status.iter_mut() {
            if let TileStatus::Known = *tile {
                *tile = TileStatus::Remembered;
            }
        }
        'outer: for i in 0..directions {
            let angle = 2.0 * PI / directions as f32 * i as f32;
            let direction = Vec2::from_angle(angle);
            let start = vec2(self.x as f32, self.y as f32) + 0.5;
            for step in 0..steps {
                let current_pos = start + step as f32 * direction;
                let (tile_x, tile_y) = (current_pos.x as usize, current_pos.y as usize);
                if dungeon.tiles[tile_x + tile_y * TILES_HORIZONTAL].is_walkable() {
                    self.tile_status[tile_x + tile_y * TILES_HORIZONTAL] = TileStatus::Known;
                } else {
                    continue 'outer;
                }
            }
        }
    }
    pub fn reset_draw_pos(&mut self) {
        self.draw_pos = vec2((self.x * 8) as f32, (self.y * 8) as f32);
    }
    pub fn center_camera(&mut self, screen_size: (f32, f32)) {
        self.camera_pos.x = self.draw_pos.x - screen_size.0 / 2.0;
        self.camera_pos.y = self.draw_pos.y - screen_size.1 / 2.0;
    }
    pub fn move_to(&mut self, pos: (usize, usize), dungeon: &Dungeon) {
        (self.x, self.y) = pos;
        self.get_visible_tiles(dungeon);
        self.reset_draw_pos();
    }
    pub fn get_visible_enemies<'a>(&self, dungeon: &'a Dungeon) -> Vec<&'a Enemy> {
        dungeon
            .enemies
            .iter()
            .filter(|f| {
                matches!(
                    self.tile_status[f.x + f.y * TILES_HORIZONTAL],
                    TileStatus::Known
                )
            })
            .collect()
    }
    pub fn update(
        &mut self,
        dungeon: &mut Dungeon,
        delta_time: f32,
        state: &GameState,
        click: Option<(usize, usize)>,
    ) -> bool {
        if let Some((tile_x, tile_y)) = click
            && dungeon.tiles[tile_x + tile_y * TILES_HORIZONTAL].is_walkable()
            && !self.tile_status[tile_x + tile_y * TILES_HORIZONTAL].is_unknown()
        {
            // if we click an enemy which is in range, attack it.
            if let Some(enemy) = dungeon
                .enemies
                .iter_mut()
                .find(|f| (f.x, f.y) == (tile_x, tile_y))
            {
                let delta = vec2(tile_x as f32 - self.x as f32, tile_y as f32 - self.y as f32);
                if self
                    .active_weapon
                    .attack_range
                    .contains(&((delta.length() - 1.0) as usize))
                {
                    self.active_action = Some(PlayerAction::Attack(delta.normalize()));
                    enemy.health -= self.active_weapon.base_damage;
                    return true;
                }
            } else {
                let goal = (tile_x, tile_y);
                let result = dungeon.pathfind((self.x, self.y), goal);
                if let Some((mut result, _)) = result {
                    result.reverse();
                    self.moving_to = result;
                }
            }
        }
        match state {
            GameState::PlayerAction(time) => {
                self.update_action(dungeon, delta_time, ACTION_TIME - *time);
                false
            }
            GameState::Idle => self.update_idle(dungeon, delta_time),
            _ => false,
        }
    }
    /// Called each frame while an action is being performed,
    /// i.e. during the 'animation' of the action
    pub fn update_action(&mut self, _dungeon: &mut Dungeon, delta_time: f32, animation_time: f32) {
        match self.active_action.as_ref().unwrap() {
            PlayerAction::MoveDirection(dir) => {
                let speed = delta_time * 8.0 / ACTION_TIME;
                let target = vec2((self.x * 8) as f32, (self.y * 8) as f32);
                if self.draw_pos.distance(target) <= speed {
                    self.draw_pos = target;
                } else {
                    self.draw_pos += *dir * speed;
                }
            }
            PlayerAction::Attack(dir) => {
                self.draw_pos = vec2((self.x * 8) as f32, (self.y * 8) as f32);
                self.draw_pos += *dir * (animation_time / ACTION_TIME * PI).sin() * 3.0;
            }
        }
    }
    /// Called each frame when no action is being performed.
    ///
    /// Returns whether player performed an action, and subsequently game should let all enemies act.
    pub fn update_idle(&mut self, dungeon: &mut Dungeon, _delta_time: f32) -> bool {
        self.reset_draw_pos();
        let input = if let Some(step) = self.moving_to.pop() {
            let x = step.0 as f32 - self.x as f32;
            let y = step.1 as f32 - self.y as f32;
            vec2(x, y)
        } else {
            get_input_axis()
        };
        if input.length() == 1.0 {
            let new = (
                self.x.saturating_add_signed(input.x as isize),
                self.y.saturating_add_signed(input.y as isize),
            );
            if let Some(enemy) = dungeon.enemies.iter_mut().find(|f| (f.x, f.y) == new) {
                // attack enemy
                if self.active_weapon.attack_range.contains(&0) {
                    self.active_action = Some(PlayerAction::Attack(input));
                    enemy.health -= self.active_weapon.base_damage;
                    return true;
                }
            } else if dungeon.tiles[new.0 + new.1 * TILES_HORIZONTAL].is_walkable() {
                (self.x, self.y) = new;
                self.get_visible_tiles(dungeon);
                self.active_action = Some(PlayerAction::MoveDirection(input));
                return true;
            }
        }

        false
    }
}

pub struct EnemyType {
    pub sprite_x: f32,
    pub sprite_y: f32,
    pub max_health: f32,
}

pub static ZOMBIE: EnemyType = EnemyType {
    sprite_x: 0.0,
    sprite_y: 3.0,
    max_health: 10.0,
};
#[expect(dead_code)]
pub static SPIDER: EnemyType = EnemyType {
    sprite_x: 0.0,
    sprite_y: 4.0,
    max_health: 6.0,
};

pub struct Enemy {
    pub x: usize,
    pub y: usize,
    pub ty: &'static EnemyType,
    pub awake: bool,
    pub just_awoke: bool,
    pub health: f32,
}
impl Enemy {
    pub fn new(x: usize, y: usize, ty: &'static EnemyType) -> Self {
        Self {
            x,
            y,
            ty,
            awake: false,
            just_awoke: false,
            health: ty.max_health,
        }
    }
    pub fn act(&mut self) {
        if self.just_awoke {
            self.just_awoke = false;
        }
    }
    pub fn draw(&self, assets: &assets::Assets, time_since_start: f64) {
        assets.tileset.draw_tile(
            (self.x * 8) as f32,
            (self.y * 8) as f32,
            self.ty.sprite_x,
            self.ty.sprite_y,
            None,
        );
        if !self.awake || self.just_awoke {
            let tile_x = if !self.awake { 3.0 } else { 4.0 };
            // modulate sleep bubble based on sin of time
            let modulate = ((time_since_start * 2.0).sin() + 1.0) as f32 * 1.5;
            // draw sleep icon
            assets.tileset.draw_tile(
                (self.x * 8) as f32 + 4.0,
                (self.y * 8) as f32 - 7.0 + modulate,
                tile_x,
                0.0,
                None,
            );
        }
    }
}
