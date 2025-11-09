use crate::{
    Tile,
    dungeon::{Dungeon, DungeonFloor},
    entities::*,
    items::*,
    loot::*,
    utils::*,
};
use macroquad::prelude::*;

pub const FIRST_FLOOR: DungeonFloor = DungeonFloor {
    rooms_area: 5 * 5 * 5,
    get_sprite: &get_tile,
    per_room_fn: &|x: usize, y: usize, w: usize, h: usize, _, enemies| {
        if rand::gen_range(0, 5) < 3 {
            enemies.push(Enemy::new(
                x + rand::gen_range(0, w),
                y + rand::gen_range(0, h),
                &ZOMBIE,
            ));
        }
    },
    post_gen_fn: &|dungeon| {
        place_random_door(dungeon);

        // generate veins of bushes
        for _ in 0..2 {
            let i = get_random_walkable(&dungeon.tiles).0;
            for (x, y) in drunkards_walk(dungeon, (i % TILES_HORIZONTAL, i / TILES_HORIZONTAL), 5) {
                dungeon.tiles[x + y * TILES_HORIZONTAL] = Tile::Chest(3.0, 1.0, &BUSH_LOOT);
            }
        }

        if rand::gen_range(0, 10) < 4 {
            println!("mushrooms!");
            let i = get_random_walkable(&dungeon.tiles).0;
            for (x, y) in drunkards_walk(dungeon, (i % TILES_HORIZONTAL, i / TILES_HORIZONTAL), 3) {
                dungeon.tiles[x + y * TILES_HORIZONTAL] = Tile::Chest(5.0, 1.0, &MUSHROOM_LOOT);
            }
        }

        let walkables = get_walkables(&dungeon.tiles);
        // place rocks
        for _ in 0..rand::gen_range(3, 6) {
            let i = walkables[rand::gen_range(0, walkables.len())].0;
            let (x, y) = (i % TILES_HORIZONTAL, i / TILES_HORIZONTAL);
            dungeon.items.push((x, y, Item::Misc(&STONE)));
        }
    },
};
pub const SECOND_FLOOR: DungeonFloor = DungeonFloor {
    per_room_fn: &|x: usize, y: usize, w: usize, h: usize, _, enemies| {
        let ty = [&ZOMBIE, &SPIDER, &SKELETON][rand::gen_range(0, 2)];
        enemies.push(Enemy::new(
            x + rand::gen_range(0, w),
            y + rand::gen_range(0, h),
            ty,
        ));
    },
    post_gen_fn: &|dungeon| {
        (FIRST_FLOOR.post_gen_fn)(dungeon);
        let mut walkables = get_walkables(&dungeon.tiles);
        let amt = rand::gen_range(0, 4);
        for _ in 0..amt {
            let rng = rand::gen_range(0, walkables.len());
            let (index, _) = walkables.remove(rng);
            let (x, y) = (index % TILES_HORIZONTAL, index / TILES_HORIZONTAL);
            if !dungeon.enemies.iter().any(|f| (f.x, f.y) == (x, y)) {
                dungeon.enemies.push(Enemy::new(x, y, &BAT));
            }
        }
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
        Tile::Chest(tile_x, tile_y, _) => (*tile_x, *tile_y),
        Tile::Detail(tile_x, tile_y) => (*tile_x, *tile_y),
    }
}

fn get_walkables<'a>(tiles: &'a [Tile]) -> Vec<(usize, &'a Tile)> {
    tiles
        .iter()
        .enumerate()
        .filter_map(|(i, f)| {
            if matches!(f, Tile::Floor) {
                Some((i, f))
            } else {
                None
            }
        })
        .collect()
}

fn get_random_walkable<'a>(tiles: &'a [Tile]) -> (usize, &'a Tile) {
    let walkables = get_walkables(tiles);
    walkables[rand::gen_range(0, walkables.len())]
}

// https://en.wikipedia.org/wiki/Random_walk
fn drunkards_walk(
    dungeon: &Dungeon,
    start: (usize, usize),
    mut tiles: usize,
) -> Vec<(usize, usize)> {
    let mut walked = Vec::new();
    let (mut x, mut y) = (start.0, start.1);
    while tiles > 0 {
        walked.push((x, y));
        let mut candidates = vec![];
        if x > 0 {
            candidates.push((x - 1, y));
        }
        if x < TILES_HORIZONTAL - 1 {
            candidates.push((x + 1, y));
        }
        if y > 0 {
            candidates.push((x, y - 1));
        }
        if y < TILES_VERTICAL - 1 {
            candidates.push((x, y + 1));
        }
        // filter away non walkable tiles
        candidates.retain(|(x, y)| {
            dungeon.tiles[x + y * TILES_HORIZONTAL].is_walkable()
                && !dungeon.enemies.iter().any(|f| (f.x, f.y) == (*x, *y))
                && (*x, *y) != dungeon.player_spawn
                && !walked.contains(&(*x, *y))
        });
        if candidates.is_empty() {
            break;
        }
        tiles -= 1;
        (x, y) = candidates[rand::gen_range(0, candidates.len())];
    }
    walked
}
fn place_random_door(dungeon: &mut Dungeon) {
    let mut walkables: Vec<(usize, &mut Tile)> = dungeon
        .tiles
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
    let player_pos = vec2(dungeon.player_spawn.0 as f32, dungeon.player_spawn.1 as f32);
    let walkables_len = walkables.len();
    const DOOR_SPAWN_ATTEMPTS: u8 = 10;
    for i in 0..DOOR_SPAWN_ATTEMPTS {
        let (index, tile) = &mut walkables[rand::gen_range(0, walkables_len)];
        let x = *index % TILES_HORIZONTAL;
        let y = *index / TILES_HORIZONTAL;
        let pos = vec2(x as f32, y as f32);
        let dist = pos.distance(player_pos);
        if i != DOOR_SPAWN_ATTEMPTS - 1
            && (dist < 10.0 || dungeon.enemies.iter().any(|f| (f.x, f.y) == (x, y)))
        {
            continue;
        }
        **tile = Tile::Door;
        break;
    }
}

#[cfg(test)]
mod tests {
    use macroquad::texture::Image;

    use crate::dungeon::{
        Dungeon,
        floors::{drunkards_walk, get_random_walkable},
    };

    #[test]
    fn test_drunkards_walk() {
        let dungeon = Dungeon::load_from_file(
            Image::from_file_with_format(include_bytes!("../../assets/testing_map.png"), None)
                .unwrap(),
        );
        let result = drunkards_walk(&dungeon, dungeon.player_spawn, 5);
        println!("{result:?}")
    }
    #[test]
    fn test_random_tile() {
        let dungeon = Dungeon::load_from_file(
            Image::from_file_with_format(include_bytes!("../../assets/testing_map.png"), None)
                .unwrap(),
        );
        get_random_walkable(&dungeon.tiles);
    }
}
