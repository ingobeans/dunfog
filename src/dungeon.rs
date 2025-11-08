use macroquad::prelude::*;
use std::iter::Map;

use crate::Tile;
use crate::entities;
use crate::utils::*;

type PerRoomFn =
    &'static dyn Fn(usize, usize, usize, usize, &mut Vec<Tile>, &mut Vec<entities::Enemy>);

type PostGenFn = &'static dyn Fn((usize, usize), &mut Vec<Tile>, &mut Vec<entities::Enemy>);

pub struct DungeonFloor {
    pub rooms_area: usize,
    pub get_sprite: &'static dyn Fn(&Tile) -> (f32, f32),
    pub per_room_fn: PerRoomFn,
    pub post_gen_fn: PostGenFn,
}

fn get_tile(tile: &Tile) -> (f32, f32) {
    #[expect(unreachable_patterns)]
    match tile {
        Tile::Floor | Tile::Path => (0.0, 1.0),
        Tile::Path => (1.0, 1.0),
        Tile::Wall => (0.0, 0.0),
        Tile::Door => (2.0, 1.0),
    }
}
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
    },
};

pub struct Dungeon {
    pub tiles: Vec<Tile>,
    pub player_spawn: (usize, usize),
    pub enemies: Vec<entities::Enemy>,
    pub dungeon_floor: &'static DungeonFloor,
}
impl Dungeon {
    pub fn load_from_file(image: Image) -> Self {
        assert_eq!(image.width, TILES_HORIZONTAL as u16);
        assert_eq!(image.height, TILES_VERTICAL as u16);
        let mut tiles = vec![Tile::Wall; TILES_HORIZONTAL * TILES_VERTICAL];
        let mut player_spawn = (0, 0);
        let mut enemies = Vec::new();
        for (index, pixel) in image.get_image_data().iter().enumerate() {
            let x = index % TILES_HORIZONTAL;
            let y = index / TILES_HORIZONTAL;
            match *pixel {
                [255, 255, 255, _] => {
                    tiles[index] = Tile::Floor;
                }
                [0, 0, 255, _] => {
                    tiles[index] = Tile::Floor;
                    player_spawn = (x, y);
                }
                [0, 255, 0, _] => {
                    tiles[index] = Tile::Floor;
                    enemies.push(entities::Enemy::new(x, y, &entities::ZOMBIE));
                }
                [0, 0, 50, _] => {
                    tiles[index] = Tile::Floor;
                    enemies.push(entities::Enemy::new(x, y, &entities::SPIDER));
                }
                [220, 220, 0, _] => {
                    tiles[index] = Tile::Floor;
                    enemies.push(entities::Enemy::new(x, y, &entities::SKELETON));
                }

                _ => {}
            }
        }
        Self {
            tiles,
            player_spawn,
            enemies,
            dungeon_floor: &FIRST_FLOOR,
        }
    }
    pub fn generate_dungeon(dungeon_floor: &'static DungeonFloor) -> Self {
        let mut enemies = Vec::new();
        let mut tiles = vec![Tile::Wall; TILES_HORIZONTAL * TILES_VERTICAL];

        let rooms_area = dungeon_floor.rooms_area;
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
            } else {
                (dungeon_floor.per_room_fn)(x, y, w, h, &mut tiles, &mut enemies)
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
        (dungeon_floor.post_gen_fn)(player_spawn.unwrap(), &mut tiles, &mut enemies);

        Self {
            tiles,
            player_spawn: player_spawn.unwrap(),
            enemies,
            dungeon_floor,
        }
    }

    pub fn pathfind(
        &self,
        from: (usize, usize),
        to: (usize, usize),
    ) -> Option<(Vec<(usize, usize)>, usize)> {
        pathfinding::prelude::astar(
            &from,
            |p| self.generate_successors(*p),
            |&(x, y)| (to.0.abs_diff(x) + to.1.abs_diff(y)) / 3,
            |&p| p == to,
        )
    }
    fn generate_successors(&self, pos: (usize, usize)) -> SuccessorIterator {
        let (x, y) = pos;
        let mut candidates = vec![(x + 1, y), (x, y + 1)];
        if x > 0 {
            candidates.push((x - 1, y));
        }
        if y > 0 {
            candidates.push((x, y - 1));
        }
        candidates.retain(|(cx, cy)| self.tiles[cx + cy * TILES_HORIZONTAL].is_walkable());
        fn map_function(p: (usize, usize)) -> ((usize, usize), usize) {
            (p, 1)
        }
        let mapped: SuccessorIterator = candidates.into_iter().map(map_function);
        mapped
    }
}
type SuccessorIterator =
    Map<std::vec::IntoIter<(usize, usize)>, fn((usize, usize)) -> ((usize, usize), usize)>;

pub fn is_all_rooms_connected(tiles: &[Tile]) -> bool {
    // algorithm:
    // 1. counts total floor tiles.
    // 2. finds first floor tile.
    // 3. recursively follows all floor tile neighbours
    //    until end and counts how many total connected tiles there are

    let total = tiles.iter().filter(|f| f.is_walkable()).count();

    fn count_connected(
        x: usize,
        y: usize,
        tiles: &[Tile],
        marked: &mut Vec<(usize, usize)>,
    ) -> usize {
        if !tiles[x + y * TILES_HORIZONTAL].is_walkable() {
            return 0;
        }
        if marked.contains(&(x, y)) {
            return 0;
        }
        marked.push((x, y));
        let mut count = 1;
        if x > 0 {
            count += count_connected(x - 1, y, tiles, marked);
        }
        if x < TILES_HORIZONTAL - 1 {
            count += count_connected(x + 1, y, tiles, marked);
        }
        if y > 0 {
            count += count_connected(x, y - 1, tiles, marked);
        }
        if y < TILES_VERTICAL - 1 {
            count += count_connected(x, y + 1, tiles, marked);
        }
        count
    }
    // find first floor tile
    for (i, tile) in tiles.iter().enumerate() {
        if tile.is_walkable() {
            let y = i / (TILES_HORIZONTAL);
            let x = i % (TILES_HORIZONTAL);
            return count_connected(x, y, tiles, &mut Vec::new()) == total;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::{Tile, dungeon::is_all_rooms_connected, utils::*};

    #[test]
    fn test_rooms_connected() {
        let mut tiles = vec![Tile::Wall; TILES_HORIZONTAL * TILES_VERTICAL];
        tiles[0] = Tile::Floor;
        tiles[1] = Tile::Floor;
        assert!(is_all_rooms_connected(&tiles))
    }
}
