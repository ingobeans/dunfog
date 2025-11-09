use std::{f32::consts::PI, sync::LazyLock};

use crate::{
    GameState, Tile, assets,
    dungeon::Dungeon,
    items::*,
    loot::{LootTable, SKELETON_DROPS},
    particles::ProjectileParticle,
    utils::*,
};
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
    pub health: f32,
    pub was_damaged: bool,
    pub should_throw_item: Option<(usize, Vec2)>,
    pub should_drop_item: Option<usize>,
    pub enemies_slayed: u32,
}
impl Default for Player {
    fn default() -> Self {
        let mut inventory = vec![None; 14];
        inventory[0] = Some(Item::Weapon(&DAGGER));
        inventory[1] = Some(Item::Armor(&IRON_ARMOR));
        inventory[2] = Some(Item::Misc(&STICK));
        inventory[3] = Some(Item::Misc(&STONE));
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
            health: MAX_PLAYER_HP,
            was_damaged: false,
            should_throw_item: None,
            should_drop_item: None,
            enemies_slayed: 0,
        }
    }
}
impl Player {
    pub fn damage(&mut self, amt: f32) {
        let rng = rand::gen_range(0.0, 1.0);
        if self.inventory[1].is_none_or(|f| {
            if let Item::Armor(armor) = f {
                armor.block_chance < rng
            } else {
                panic!()
            }
        }) {
            self.was_damaged = true;
            self.health -= amt;
        }
    }
    pub fn draw(&self, assets: &assets::Assets, _time_since_start: f64) {
        if self.was_damaged {
            gl_use_material(&DAMAGE_MATERIAL);
        }
        assets
            .tileset
            .draw_tile(self.draw_pos.x, self.draw_pos.y, 0.0, 2.0, None);
        if self.was_damaged {
            gl_use_default_material();
        }
        if let Some(Item::Weapon(item)) = &self.inventory[0] {
            assets.items.draw_tile(
                self.draw_pos.x - 5.0,
                self.draw_pos.y - 2.0,
                item.sprite_x,
                item.sprite_y,
                None,
            );
        }
        if let Some(Item::Armor(item)) = &self.inventory[1] {
            assets.items.draw_tile(
                self.draw_pos.x,
                self.draw_pos.y,
                item.sprite_x + 1.0,
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
        // stop moving towards target if any key is pressed
        if !self.moving_to.is_empty() && !get_keys_pressed().is_empty() {
            self.moving_to = Vec::new();
        }
        if let GameState::Idle = state
            && let Some((index, pos)) = self.should_throw_item
        {
            let item = self.inventory[index].take().unwrap();
            self.should_throw_item = None;
            let self_pos = vec2(self.x as f32, self.y as f32);
            let delta_normalized = (pos - self_pos).normalize();
            let mut current = self_pos;
            let max_step = 0.15;
            loop {
                current += delta_normalized * max_step;
                let (tx, ty) = ((current.x).round() as usize, (current.y).round() as usize);
                if !dungeon.tiles[tx + ty * TILES_HORIZONTAL].is_walkable() {
                    break;
                }
                if let Some(enemy) = dungeon.enemies.iter_mut().find(|f| f.x == tx && f.y == ty) {
                    let throwable = item.throwable().unwrap();
                    enemy.damage(throwable.0);
                    let dest = pos * 8.0 + 4.0;
                    dungeon.particles.push(Box::new(ProjectileParticle {
                        sprite_x: throwable.1.x,
                        sprite_y: throwable.1.y,
                        origin: self.draw_pos + 4.0,
                        dest,
                    }));
                    break;
                }
            }

            return Some(PlayerAction::Attack(delta_normalized));
        }
        if let GameState::Idle = state
            && let Some((tile_x, tile_y)) = click
            && dungeon.tiles[tile_x + tile_y * TILES_HORIZONTAL].is_walkable()
            && !self.tile_status[tile_x + tile_y * TILES_HORIZONTAL].is_unknown()
        {
            let delta = vec2(tile_x as f32 - self.x as f32, tile_y as f32 - self.y as f32);
            let Item::Weapon(weapon) = &self.inventory[0].unwrap_or(Item::Weapon(&MELEE)) else {
                panic!("non weapon-type item in weapon slot")
            };
            let weapon_in_range =
                ((delta.length()) as usize) <= weapon.attack_range.clone().max().unwrap();

            // if we click an enemy which is in range, attack it.
            if let Some(enemy) = dungeon
                .enemies
                .iter_mut()
                .find(|f| (f.x, f.y) == (tile_x, tile_y))
                && (weapon_in_range
                    || matches!(
                        self.tile_status[tile_x + tile_y * TILES_HORIZONTAL],
                        TileStatus::Known
                    ))
            {
                if weapon_in_range {
                    enemy.damage(weapon.base_damage);
                    if let Some(particle) = weapon.fires_particle {
                        dungeon.particles.push(Box::new(ProjectileParticle {
                            sprite_x: particle.0,
                            sprite_y: particle.1,
                            origin: self.draw_pos + 4.0,
                            dest: vec2(tile_x as f32 * 8.0 + 4.0, tile_y as f32 * 8.0 + 4.0),
                        }));
                    }
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
        if self.was_damaged {
            self.was_damaged = false;
        }
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
            if !dungeon.enemies.iter_mut().any(|f| (f.x, f.y) == new)
                && dungeon.tiles[new.0 + new.1 * TILES_HORIZONTAL].is_walkable()
            {
                (self.x, self.y) = new;
                self.get_visible_tiles(dungeon);
                return Some(PlayerAction::MoveDirection(input));
            }
        }
        if is_key_pressed(KeyCode::H) {
            return Some(PlayerAction::Wait);
        }
        if is_key_pressed(KeyCode::E) {
            // try interact with current tile
            let tile = &mut dungeon.tiles[self.x + self.y * TILES_HORIZONTAL];
            return match tile {
                Tile::Chest(sprite_x, sprite_y, _) => {
                    let mut buffer = Tile::Detail(*sprite_x + 1.0, *sprite_y);
                    std::mem::swap(&mut buffer, tile);
                    if let Tile::Chest(_, _, loot) = buffer {
                        if let Some(item) = loot.get_item() {
                            if let Some(slot) = self.get_free_slot() {
                                self.inventory[slot] = Some(*item);
                            } else {
                                dungeon.items.push((self.x, self.y, *item));
                            }
                        }
                    }

                    None
                }
                Tile::Door => Some(PlayerAction::GotoNextDungeon),
                _ => {
                    if let Some(item) = dungeon
                        .items
                        .iter()
                        .position(|(x, y, _)| (x, y) == (&self.x, &self.y))
                        && let Some(slot) = self.get_free_slot()
                    {
                        let (_, _, item) = dungeon.items.remove(item);
                        self.inventory[slot] = Some(item);
                    }
                    None
                }
            };
        }

        None
    }
    pub fn get_free_slot(&self) -> Option<usize> {
        for (i, slot) in self.inventory.iter().enumerate().skip(2) {
            if slot.is_none() {
                return Some(i);
            }
        }
        None
    }
}

pub enum MovementType {
    ChaseWhenVisible,
    AlwaysChase,
}

pub struct EnemyType {
    pub block_chance: f32,
    pub death_drops: Option<&'static LootTable>,
    pub sprite_x: f32,
    pub sprite_y: f32,
    pub max_health: f32,
    pub movement_type: MovementType,
    pub weapon: &'static Weapon,
    pub show_held_item: bool,
}

pub static ZOMBIE: EnemyType = EnemyType {
    block_chance: 0.1,
    death_drops: None,
    sprite_x: 0.0,
    sprite_y: 3.0,
    max_health: 10.0,
    movement_type: MovementType::ChaseWhenVisible,
    weapon: &MELEE,
    show_held_item: false,
};
pub static SKELETON: LazyLock<EnemyType> = LazyLock::new(|| EnemyType {
    block_chance: 0.1,
    death_drops: Some(&SKELETON_DROPS),
    sprite_x: 0.0,
    sprite_y: 5.0,
    max_health: 10.0,
    movement_type: MovementType::AlwaysChase,
    weapon: &BOW,
    show_held_item: false,
});
pub static SPIDER: EnemyType = EnemyType {
    block_chance: 0.5,
    death_drops: None,
    sprite_x: 0.0,
    sprite_y: 4.0,
    max_health: 6.0,
    movement_type: MovementType::AlwaysChase,
    weapon: &MELEE,
    show_held_item: false,
};
pub static BAT: EnemyType = EnemyType {
    block_chance: 0.8,
    death_drops: None,
    sprite_x: 0.0,
    sprite_y: 5.0,
    max_health: 6.0,
    movement_type: MovementType::ChaseWhenVisible,
    weapon: &MELEE,
    show_held_item: false,
};
pub static SLIME: EnemyType = EnemyType {
    block_chance: 0.0,
    death_drops: None,
    sprite_x: 0.0,
    sprite_y: 6.0,
    max_health: 16.0,
    movement_type: MovementType::ChaseWhenVisible,
    weapon: &MELEE,
    show_held_item: false,
};
pub static WIZARD: EnemyType = EnemyType {
    block_chance: 0.2,
    death_drops: None,
    sprite_x: 0.0,
    sprite_y: 7.0,
    max_health: 10.0,
    movement_type: MovementType::AlwaysChase,
    weapon: &SPELLBOOK,
    show_held_item: false,
};
pub static LAVA_DOG: EnemyType = EnemyType {
    block_chance: 0.6,
    death_drops: None,
    sprite_x: 0.0,
    sprite_y: 8.0,
    max_health: 5.0,
    movement_type: MovementType::AlwaysChase,
    weapon: &MELEE,
    show_held_item: false,
};

pub enum EnemyAction {
    MoveTo((usize, usize)),
    Attack(Vec2),
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
    pub was_damaged: bool,
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
            was_damaged: false,
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
        if let GameState::EnemyAction(time) = state
            && let Some(current_action) = &self.current_action
        {
            let animation_time = ACTION_TIME - *time;
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
                EnemyAction::Attack(dir) => {
                    self.draw_pos = vec2((self.x * 8) as f32, (self.y * 8) as f32);
                    self.draw_pos += *dir * (animation_time / ACTION_TIME * PI).sin() * 3.0;
                }
            }
        } else {
            self.reset_draw_pos();
        }
    }
    pub fn damage(&mut self, amt: f32) {
        let rng = rand::gen_range(0.0, 1.0);
        if self.ty.block_chance < rng {
            self.was_damaged = true;
            self.health -= amt;
        }
    }
    pub fn act(&mut self, dungeon: &mut Dungeon, player: &mut Player) -> EnemyAction {
        if self.just_awoke {
            self.just_awoke = false;
        }
        if self.was_damaged {
            self.was_damaged = false;
        }
        let delta = vec2(
            player.x as f32 - self.x as f32,
            player.y as f32 - self.y as f32,
        );
        if self
            .ty
            .weapon
            .attack_range
            .contains(&((delta.length()) as usize))
        {
            player.damage(self.ty.weapon.base_damage);

            if let Some(particle) = self.ty.weapon.fires_particle {
                dungeon.particles.push(Box::new(ProjectileParticle {
                    sprite_x: particle.0,
                    sprite_y: particle.1,
                    origin: self.draw_pos + 4.0,
                    dest: vec2(player.x as f32 * 8.0 + 4.0, player.y as f32 * 8.0 + 4.0),
                }));
            }
            return EnemyAction::Attack(delta.normalize());
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
            let dist = delta.length();

            let min_range = self.ty.weapon.attack_range.clone().min().unwrap_or(0);
            let max_range = self.ty.weapon.attack_range.clone().max().unwrap_or(0);

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
        if self.was_damaged {
            gl_use_material(&DAMAGE_MATERIAL);
        }
        assets.tileset.draw_tile(
            self.draw_pos.x,
            self.draw_pos.y,
            self.ty.sprite_x,
            self.ty.sprite_y,
            None,
        );
        if self.was_damaged {
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
        if self.ty.show_held_item {
            assets.items.draw_tile(
                self.draw_pos.x - 4.0,
                self.draw_pos.y - 2.0,
                self.ty.weapon.sprite_x,
                self.ty.weapon.sprite_y,
                None,
            );
        }
    }
}
