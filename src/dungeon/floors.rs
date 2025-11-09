use crate::{Tile, dungeon::DungeonFloor, entities, utils::*};
use macroquad::prelude::*;

pub const FIRST_FLOOR: DungeonFloor = DungeonFloor {
    rooms_area: 5 * 5 * 5,
    get_sprite: &get_tile,
    per_room_fn: &|x: usize, y: usize, w: usize, h: usize, _, enemies| {
        enemies.push(entities::Enemy::new(
            x + rand::gen_range(0, w),
            y + rand::gen_range(0, h),
            &entities::ZOMBIE,
        ));
    },
    post_gen_fn: &|(player_x, player_y), tiles, enemies| {
        place_random_door(player_x, player_y, tiles, enemies);
    },
};
pub const SECOND_FLOOR: DungeonFloor = DungeonFloor {
    per_room_fn: &|x: usize, y: usize, w: usize, h: usize, _, enemies| {
        let ty = [&entities::ZOMBIE, &entities::SPIDER, &entities::SKELETON][rand::gen_range(0, 2)];
        enemies.push(entities::Enemy::new(
            x + rand::gen_range(0, w),
            y + rand::gen_range(0, h),
            ty,
        ));
    },
    ..FIRST_FLOOR
};

fn get_tile(tile: &Tile) -> (f32, f32) {
    #[expect(unreachable_patterns)]
    match tile {
        Tile::Floor | Tile::Path => (0.0, 1.0),
        Tile::Path => (1.0, 1.0),
        Tile::Wall => (0.0, 0.0),
        Tile::Door => (2.0, 1.0),
    }
}
fn place_random_door(
    player_x: usize,
    player_y: usize,
    tiles: &mut Vec<Tile>,
    enemies: &mut Vec<entities::Enemy>,
) {
    let mut walkables: Vec<(usize, &mut Tile)> = tiles
        .iter_mut()
        .enumerate()
        .filter_map(|(i, f)| {
            if matches!(f, Tile::Floor) {
                Some((i, f))
            } else {
                None
            }
        })
        .collect();
    let player_pos = vec2(player_x as f32, player_y as f32);
    let walkables_len = walkables.len();
    const DOOR_SPAWN_ATTEMPTS: u8 = 10;
    for i in 0..DOOR_SPAWN_ATTEMPTS {
        let (index, tile) = &mut walkables[rand::gen_range(0, walkables_len)];
        let x = *index % TILES_HORIZONTAL;
        let y = *index / TILES_HORIZONTAL;
        let pos = vec2(x as f32, y as f32);
        let dist = pos.distance(player_pos);
        if i != DOOR_SPAWN_ATTEMPTS - 1
            && (dist < 10.0 || enemies.iter().any(|f| (f.x, f.y) == (x, y)))
        {
            continue;
        }
        **tile = Tile::Door;
        break;
    }
}
