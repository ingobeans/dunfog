#![expect(irrefutable_let_patterns)]
use std::f32::consts::PI;

use crate::{GameState, Tile, assets, dungeon::Dungeon, utils::*};
use macroquad::prelude::*;

pub enum PlayerAction {
    MoveDirection(Vec2),
    Attack(Vec2),
    Wait,
    GotoNextDungeon,
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
    pub sprite_x: f32,
    pub sprite_y: f32,
}
pub const MELEE: Weapon = Weapon {
    attack_range: 0..1,
    base_damage: 1.0,
    sprite_x: 0.0,
    sprite_y: 0.0,
};
pub const DAGGER: Weapon = Weapon {
    attack_range: 0..1,
    base_damage: 2.0,
    sprite_x: 1.0,
    sprite_y: 0.0,
};
pub const BOW: Weapon = Weapon {
    attack_range: 1..3,
    base_damage: 2.0,
    sprite_x: 2.0,
    sprite_y: 0.0,
};
#[derive(Clone, Copy)]
pub enum Item {
    Weapon(&'static Weapon),
}
impl Item {
    pub fn get_sprite(&self) -> Vec2 {
        match &self {
            Item::Weapon(weapon) => vec2(weapon.sprite_x, weapon.sprite_y),
        }
    }
    // fn unwrap_weapon(&self) -> &'static Weapon {
    //     match &self {
    //         Item::Weapon(weapon) => &weapon,
    //         _ => {
    //             panic!()
    //         }
    //     }
    // }
}

pub struct Player {
    pub active_action: Option<PlayerAction>,
    pub moving_to: Vec<(usize, usize)>,
    pub x: usize,
    pub y: usize,
    pub draw_pos: Vec2,
    pub camera_pos: Vec2,
    pub camera_zoom: f32,
    pub tile_status: Vec<TileStatus>,
    pub inventory: Vec<Option<Item>>,
    pub cursor_item: Option<Item>,
}
impl Default for Player {
    fn default() -> Self {
        let mut inventory = vec![None; 14];
        inventory[0] = Some(Item::Weapon(&DAGGER));
        Self {
            active_action: None,
            moving_to: Vec::new(),
            x: 0,
            y: 0,
            draw_pos: Vec2::ZERO,
            camera_pos: vec2(0.0, 0.0),
            camera_zoom: 1.0,
            tile_status: vec![TileStatus::Unknown; TILES_HORIZONTAL * TILES_VERTICAL],
            inventory,
            cursor_item: None,
        }
    }
}
impl Player {
    pub fn draw(&self, assets: &assets::Assets, _time_since_start: f64) {
        assets
            .tileset
            .draw_tile(self.draw_pos.x, self.draw_pos.y, 0.0, 2.0, None);
        if let Some(Item::Weapon(item)) = &self.inventory[0] {
            assets.items.draw_tile(
                self.draw_pos.x - 4.0,
                self.draw_pos.y - 2.0,
                item.sprite_x,
                item.sprite_y,
                None,
            );
        }
    }
    /// Uses raycasts to see which tiles should be visible by player
    pub fn get_visible_tiles(&mut self, dungeon: &Dungeon) {
        let directions = 20;
        let steps = 5;
        let substeps = 5;

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
                for substep in 0..substeps {
                    let substep = substep + 1;
                    let current_pos =
                        start + (step as f32 * direction * substep as f32 / substeps as f32);
                    let (tile_x, tile_y) = (current_pos.x as usize, current_pos.y as usize);
                    if dungeon.tiles[tile_x + tile_y * TILES_HORIZONTAL].is_walkable() {
                        self.tile_status[tile_x + tile_y * TILES_HORIZONTAL] = TileStatus::Known;
                    } else {
                        continue 'outer;
                    }
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
    ) -> Option<PlayerAction> {
        if let Tile::Door = dungeon.tiles[self.x + self.y * TILES_HORIZONTAL]
            && let GameState::Idle = state
        {
            return Some(PlayerAction::GotoNextDungeon);
        }
        if let Some((tile_x, tile_y)) = click
            && dungeon.tiles[tile_x + tile_y * TILES_HORIZONTAL].is_walkable()
            && !self.tile_status[tile_x + tile_y * TILES_HORIZONTAL].is_unknown()
        {
            // if we click an enemy which is in range, attack it.
            if let GameState::Idle = state
                && let Some(enemy) = dungeon
                    .enemies
                    .iter_mut()
                    .find(|f| (f.x, f.y) == (tile_x, tile_y))
            {
                let delta = vec2(tile_x as f32 - self.x as f32, tile_y as f32 - self.y as f32);
                if let Item::Weapon(weapon) = &self.inventory[0].unwrap_or(Item::Weapon(&MELEE))
                    && weapon
                        .attack_range
                        .contains(&((delta.length() - 1.0) as usize))
                {
                    enemy.health -= weapon.base_damage;
                    enemy.was_attacked = true;
                    return Some(PlayerAction::Attack(delta.normalize()));
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
                None
            }
            GameState::Idle => self.update_idle(dungeon, delta_time),
            _ => None,
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
            _ => {}
        }
    }
    /// Called each frame when no action is being performed.
    ///
    /// Returns whether player performed an action, and subsequently game should let all enemies act.
    pub fn update_idle(&mut self, dungeon: &mut Dungeon, _delta_time: f32) -> Option<PlayerAction> {
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
                if let Item::Weapon(weapon) = &self.inventory[0].unwrap_or(Item::Weapon(&MELEE))
                    && weapon.attack_range.contains(&0)
                {
                    enemy.health -= weapon.base_damage;
                    enemy.was_attacked = true;
                    return Some(PlayerAction::Attack(input));
                }
            } else if dungeon.tiles[new.0 + new.1 * TILES_HORIZONTAL].is_walkable() {
                (self.x, self.y) = new;
                self.get_visible_tiles(dungeon);
                return Some(PlayerAction::MoveDirection(input));
            }
        }
        if is_key_pressed(KeyCode::H) {
            return Some(PlayerAction::Wait);
        }

        None
    }
}

pub enum MovementType {
    ChaseWhenVisible,
    AlwaysChase,
}

pub struct EnemyType {
    pub sprite_x: f32,
    pub sprite_y: f32,
    pub max_health: f32,
    pub movement_type: MovementType,
    pub weapon: &'static Weapon,
}

pub static ZOMBIE: EnemyType = EnemyType {
    sprite_x: 0.0,
    sprite_y: 3.0,
    max_health: 10.0,
    movement_type: MovementType::ChaseWhenVisible,
    weapon: &MELEE,
};
pub static SKELETON: EnemyType = EnemyType {
    sprite_x: 0.0,
    sprite_y: 5.0,
    max_health: 10.0,
    movement_type: MovementType::AlwaysChase,
    weapon: &BOW,
};
pub static SPIDER: EnemyType = EnemyType {
    sprite_x: 0.0,
    sprite_y: 4.0,
    max_health: 6.0,
    movement_type: MovementType::AlwaysChase,
    weapon: &MELEE,
};

pub enum EnemyAction {
    MoveTo((usize, usize)),
    Wait,
}

pub struct Enemy {
    pub x: usize,
    pub y: usize,
    /// everyone has a favorite angle
    pub favorite_angle: f32,
    pub draw_pos: Vec2,
    pub ty: &'static EnemyType,
    pub awake: bool,
    pub just_awoke: bool,
    pub was_attacked: bool,
    pub health: f32,
    pub current_action: Option<EnemyAction>,
    #[allow(dead_code)]
    last_pathfind_target: Option<Vec2>,
}
impl Enemy {
    pub fn new(x: usize, y: usize, ty: &'static EnemyType) -> Self {
        Self {
            x,
            y,
            favorite_angle: rand::gen_range(0.0, 2.0 * PI),
            draw_pos: vec2(x as f32 * 8.0, y as f32 * 8.0),
            ty,
            awake: false,
            just_awoke: false,
            was_attacked: false,
            health: ty.max_health,
            current_action: None,
            last_pathfind_target: None,
        }
    }
    pub fn reset_draw_pos(&mut self) {
        self.draw_pos = vec2((self.x * 8) as f32, (self.y * 8) as f32);
    }
    pub fn awaken(&mut self) {
        self.awake = true;
        self.just_awoke = true;
    }
    pub fn update(&mut self, delta_time: f32, state: &GameState) {
        if let GameState::EnemyAction(_time) = state
            && let Some(current_action) = &self.current_action
        {
            match current_action {
                EnemyAction::Wait => {}
                EnemyAction::MoveTo(pos) => {
                    let speed = delta_time * 8.0 / ACTION_TIME;
                    let target = vec2(pos.0 as f32 * 8.0, pos.1 as f32 * 8.0);
                    let delta = target - self.draw_pos;
                    if self.draw_pos.distance(target) <= speed {
                        self.draw_pos = target;
                    } else {
                        self.draw_pos += delta.normalize() * speed;
                    }
                }
            }
        } else {
            self.reset_draw_pos();
        }
    }
    pub fn act(&mut self, dungeon: &Dungeon, player: &mut Player) -> EnemyAction {
        if self.just_awoke {
            self.just_awoke = false;
        }
        if self.was_attacked {
            self.was_attacked = false;
        }
        let should_pathfind = match &self.ty.movement_type {
            MovementType::ChaseWhenVisible
                if matches!(
                    player.tile_status[self.x + self.y * TILES_HORIZONTAL],
                    TileStatus::Known
                ) =>
            {
                true
            }
            MovementType::AlwaysChase => true,
            _ => false,
        };
        if should_pathfind {
            // decide whether to move away from or towards player.
            // usually it moves towards player, but if weapon min range is larger that the dist to player, move away
            let delta = vec2(
                player.x as f32 - self.x as f32,
                player.y as f32 - self.y as f32,
            );
            let dist = delta.length();

            let min_range = self.ty.weapon.attack_range.clone().min().unwrap_or(0) + 1;
            let max_range = self.ty.weapon.attack_range.clone().max().unwrap_or(0) + 1;

            let mut target_radius = if dist < min_range as f32 {
                min_range as f32
            } else if dist > max_range as f32 {
                max_range as f32
            } else {
                dist as f32
            };
            const MAX_PATHFIND_ATTEMPTS: u8 = 5;
            let mut adjusted = false;

            for _ in 0..MAX_PATHFIND_ATTEMPTS {
                let target = vec2(player.x as f32 + 0.5, player.y as f32 + 0.5)
                    + Vec2::from_angle(self.favorite_angle) * target_radius;
                if !adjusted
                    && target
                        .floor()
                        .distance(vec2(player.x as f32, player.y as f32))
                        < min_range as f32
                {
                    adjusted = true;
                    target_radius += 1.0;
                    continue;
                } else if !adjusted
                    && target
                        .floor()
                        .distance(vec2(player.x as f32, player.y as f32))
                        .floor()
                        > max_range as f32
                {
                    dbg!(
                        target
                            .distance(vec2(player.x as f32, player.y as f32))
                            .floor()
                    );
                    adjusted = true;
                    target_radius -= 1.0;
                    continue;
                }
                let target_usize = (target.x as usize, target.y as usize);
                if target_usize == (self.x, self.y) {
                    return EnemyAction::Wait;
                }

                let path = dungeon.pathfind((self.x, self.y), target_usize);
                if let Some((path, _)) = path
                    && let Some(next) = path.get(1)
                {
                    return EnemyAction::MoveTo(*next);
                } else {
                    self.favorite_angle =
                        delta.to_angle() + PI + rand::gen_range(-PI / 2.0, PI / 2.0);
                }
            }
        }
        EnemyAction::Wait
    }
    pub fn draw(&self, assets: &assets::Assets, time_since_start: f64) {
        if self.was_attacked {
            gl_use_material(&DAMAGE_MATERIAL);
        }
        assets.tileset.draw_tile(
            self.draw_pos.x,
            self.draw_pos.y,
            self.ty.sprite_x,
            self.ty.sprite_y,
            None,
        );
        if self.was_attacked {
            gl_use_default_material();
        }
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
        assets.items.draw_tile(
            self.draw_pos.x - 4.0,
            self.draw_pos.y - 2.0,
            self.ty.weapon.sprite_x,
            self.ty.weapon.sprite_y,
            None,
        );

        // if let Some(target) = &self.last_pathfind_target {
        //     draw_rectangle_lines(target.x * 8.0, target.y * 8.0, 8.0, 8.0, 2.0, RED);
        // }
        // let angle_vec = Vec2::from_angle(self.favorite_angle);
        // draw_line(
        //     self.draw_pos.x + 4.0,
        //     self.draw_pos.y + 4.0,
        //     self.draw_pos.x + 4.0 + angle_vec.x * 12.0,
        //     self.draw_pos.y + 4.0 + angle_vec.y * 12.0,
        //     2.0,
        //     YELLOW,
        // );
    }
}
