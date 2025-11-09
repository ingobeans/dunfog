use macroquad::prelude::*;
use std::iter::Map;

use crate::Tile;
use crate::entities::*;
use crate::items::Item;
use crate::loot::*;
use crate::particles::Particle;
use crate::particles::ScreenParticle;
use crate::utils::*;
use floors::*;

mod floors;

pub const DUNGEON_FLOORS: &[DungeonFloor] = &[
    FIRST_FLOOR,
    SECOND_FLOOR,
    THIRD_FLOOR,
    FOURTH_FLOOR,
    FIFTH_FLOOR,
];

type PerRoomFn = &'static dyn Fn(usize, usize, usize, usize, &mut Vec<Tile>, &mut Vec<Enemy>);

type PostGenFn = &'static dyn Fn(&mut Dungeon);

pub struct DungeonFloor {
    pub from_file: Option<&'static [u8]>,
    pub rooms_area: usize,
    pub get_sprite: &'static dyn Fn(&Tile) -> (f32, f32),
    pub per_room_fn: PerRoomFn,
    pub post_gen_fn: PostGenFn,
}

pub struct Dungeon {
    pub tiles: Vec<Tile>,
    pub player_spawn: (usize, usize),
    pub enemies: Vec<Enemy>,
    pub particles: Vec<Box<dyn Particle>>,
    pub screen_particles: Vec<Box<dyn ScreenParticle>>,
    pub items: Vec<(usize, usize, Item)>,
    pub dungeon_floor: &'static DungeonFloor,
}
impl Dungeon {
    pub fn load_from_file(image: Image) -> Self {
        assert_eq!(image.width, TILES_HORIZONTAL as u16);
        assert_eq!(image.height, TILES_VERTICAL as u16);
        let mut tiles = vec![Tile::Wall; TILES_HORIZONTAL * TILES_VERTICAL];
        let mut player_spawn = (0, 0);
        let mut enemies = Vec::new();
        let items = Vec::new();
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
                    enemies.push(Enemy::new(x, y, &ZOMBIE));
                }
                [0, 0, 50, _] => {
                    tiles[index] = Tile::Floor;
                    enemies.push(Enemy::new(x, y, &SPIDER));
                }
                [220, 220, 0, _] => {
                    tiles[index] = Tile::Floor;
                    enemies.push(Enemy::new(x, y, &SKELETON));
                }
                [200, 0, 255, _] => {
                    tiles[index] = Tile::Floor;
                    enemies.push(Enemy::new(x, y, &WIZARD));
                }
                [255, 0, 255, _] => {
                    tiles[index] = Tile::Floor;
                    enemies.push(Enemy::new(x, y, &SUPER_WIZARD));
                }
                [255, 0, 100, _] => {
                    tiles[index] = Tile::Chest(5.0, 1.0, &MUSHROOM_LOOT);
                }
                [225, 150, 100, _] => {
                    tiles[index] = Tile::Ore(7.0, 1.0, &IRON_LOOT);
                }

                _ => {}
            }
        }
        Self {
            tiles,
            player_spawn,
            enemies,
            items,
            dungeon_floor: &FIRST_FLOOR,
            particles: Vec::new(),
            screen_particles: Vec::new(),
        }
    }
    pub fn generate_dungeon(dungeon_floor: &'static DungeonFloor) -> Self {
        if let Some(bytes) = dungeon_floor.from_file {
            let image = Image::from_file_with_format(bytes, None).unwrap();
            return Self::load_from_file(image);
        }
        let mut enemies = Vec::new();
        let items = Vec::new();
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
        let mut dungeon = Self {
            tiles,
            player_spawn: player_spawn.unwrap(),
            enemies,
            items,
            dungeon_floor,
            particles: Vec::new(),
            screen_particles: Vec::new(),
        };
        (dungeon_floor.post_gen_fn)(&mut dungeon);
        dungeon
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
