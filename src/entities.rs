use crate::{assets, dungeon::Dungeon, utils::*};
use macroquad::prelude::*;

pub enum PlayerAction {
    MoveDirection(Vec2),
}

pub struct Player {
    pub active_action: Option<PlayerAction>,
    pub x: usize,
    pub y: usize,
    pub draw_pos: Vec2,
    pub camera_pos: Vec2,
    pub camera_zoom: f32,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            active_action: None,
            x: 0,
            y: 0,
            draw_pos: Vec2::ZERO,
            camera_pos: vec2(0.0, 0.0),
            camera_zoom: 1.0,
        }
    }
}
impl Player {
    pub fn draw(&self, assets: &assets::Assets, _time_since_start: f64) {
        assets
            .tileset
            .draw_tile(self.draw_pos.x, self.draw_pos.y, 0.0, 2.0, None);
    }
    pub fn move_to(&mut self, pos: (usize, usize)) {
        (self.x, self.y) = pos;
        self.draw_pos = vec2((self.x * 8) as f32, (self.y * 8) as f32);
    }
    /// Called each frame while an action is being performed,
    /// i.e. during the 'animation' of the action
    pub fn update_action(
        &mut self,
        _dungeon: &mut Dungeon,
        delta_time: f32,
        animation_length: f32,
    ) {
        assert!(self.active_action.is_some());
        match self.active_action.as_ref().unwrap() {
            PlayerAction::MoveDirection(dir) => {
                let speed = delta_time * 8.0 / animation_length;
                let target = vec2((self.x * 8) as f32, (self.y * 8) as f32);
                if self.draw_pos.distance(target) <= speed {
                    self.draw_pos = target;
                } else {
                    self.draw_pos += *dir * speed;
                }
            }
        }
    }
    /// Called each frame when no action is being performed.
    ///
    /// Returns whether player performed an action, and subsequently game should let all enemies act.
    pub fn update_idle(&mut self, dungeon: &mut Dungeon, _delta_time: f32) -> bool {
        let input = get_input_axis();
        if input.length() == 1.0 {
            let new = (
                self.x.saturating_add_signed(input.x as isize),
                self.y.saturating_add_signed(input.y as isize),
            );
            if dungeon.tiles[new.0 + new.1 * TILES_HORIZONTAL].is_walkable() {
                (self.x, self.y) = new;
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
    pub health: f32,
}
impl Enemy {
    pub fn new(x: usize, y: usize, ty: &'static EnemyType) -> Self {
        Self {
            x,
            y,
            ty,
            awake: false,
            health: ty.max_health,
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
        if !self.awake {
            // modulate sleep bubble based on sin of time
            let modulate = ((time_since_start * 2.0).sin() + 1.0) as f32 * 1.5;
            // draw sleep icon
            assets.tileset.draw_tile(
                (self.x * 8) as f32 + 4.0,
                (self.y * 8) as f32 - 7.0 + modulate,
                1.0,
                0.0,
                None,
            );
        }
    }
}
