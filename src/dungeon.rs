use macroquad::prelude::*;
use std::iter::Map;

use crate::Tile;
use crate::entities;
use crate::utils::*;

pub struct Dungeon {
    pub tiles: Vec<Tile>,
    pub player_spawn: (usize, usize),
    pub enemies: Vec<entities::Enemy>,
}
impl Dungeon {
    pub fn generate_dungeon() -> Self {
        let mut enemies = Vec::new();
        let mut tiles = vec![Tile::Wall; TILES_HORIZONTAL * TILES_VERTICAL];

        let rooms_area = 5 * 5 * 5;
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
                enemies.push(entities::Enemy::new(
                    x + rand::gen_range(0, w),
                    y + rand::gen_range(0, h),
                    &entities::ZOMBIE,
                ));
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

        Self {
            tiles,
            player_spawn: player_spawn.unwrap(),
            enemies,
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
    fn generate_successors(
        &self,
        pos: (usize, usize),
    ) -> Map<std::vec::IntoIter<(usize, usize)>, fn((usize, usize)) -> ((usize, usize), usize)>
    {
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
        let mapped: Map<
            std::vec::IntoIter<(usize, usize)>,
            fn((usize, usize)) -> ((usize, usize), usize),
        > = candidates.into_iter().map(map_function);
        mapped
    }
}
