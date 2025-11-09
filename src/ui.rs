use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::{
    Tile,
    assets::Assets,
    dungeon::Dungeon,
    entities::Player,
    items::{Item, combine, get_combinable},
    utils::*,
};

const UI_BACKGROUND: Color = Color::from_hex(0x8b9bb4);
const UI_BORDER: Color = Color::from_hex(0xc0cbdc);

pub enum InventoryAction {
    None,
    CtxMenuOpen(usize, f32, f32),
    MovingItem(usize),
    CombiningItem(usize, Vec<usize>),
}

pub enum InventoryState {
    Closed,
    Inventory(InventoryAction),
    ThrowingItem(usize),
}
impl InventoryState {
    pub fn toggle(&mut self) {
        *self = match self {
            InventoryState::Closed => InventoryState::Inventory(InventoryAction::None),
            InventoryState::Inventory(_) => InventoryState::Closed,
            InventoryState::ThrowingItem(_) => InventoryState::Closed,
        }
    }
}
pub fn draw_tooltip(text: &str, assets: &Assets) {
    let (actual_screen_width, actual_screen_height) = screen_size();
    let scale_factor = (actual_screen_width / SCREEN_WIDTH)
        .min(actual_screen_height / SCREEN_HEIGHT)
        .floor()
        .max(1.0);
    let x = (actual_screen_width - assets.tooltip.width() * scale_factor) / 2.0;
    let y = actual_screen_height - assets.tooltip.height() * scale_factor - 4.0 * scale_factor;
    draw_texture_ex(
        &assets.tooltip,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(
                assets.tooltip.width() * scale_factor,
                assets.tooltip.height() * scale_factor,
            )),
            ..Default::default()
        },
    );
    draw_text_ex(
        text,
        x + 3.0 * scale_factor,
        y + 9.0 * scale_factor,
        TextParams {
            font: Some(&assets.font),
            font_size: (scale_factor * 8.0) as u16,
            ..Default::default()
        },
    );
}
#[expect(dead_code)]
pub fn ui_rect(x: f32, y: f32, w: f32, h: f32) {
    draw_rectangle(x, y, w, h, UI_BORDER);
    draw_rectangle(x + 1.0, y + 1.0, w - 2.0, h - 2.0, UI_BACKGROUND);
}
fn slot_index_position(index: usize) -> (usize, usize) {
    let melee_slot = (32, 17);
    let inventory_start = (4, 45);
    match index {
        0 => melee_slot,
        1 => (melee_slot.0, melee_slot.1 + 13),
        2..14 => {
            let index = index - 2;
            let x = index % 4;
            let y = index / 4;
            (inventory_start.0 + x * 13, inventory_start.1 + y * 13)
        }
        _ => {
            panic!("we don't have that many slots :(")
        }
    }
}
pub fn draw_item_hover_info(
    item: &Item,
    assets: &Assets,
    mouse_x: f32,
    mouse_y: f32,
    scale_factor: f32,
) {
    let x = mouse_x - assets.hover_card.width() * scale_factor + 2.0 * scale_factor;
    let y = mouse_y - assets.hover_card.height() * scale_factor + 2.0 * scale_factor;
    draw_texture_ex(
        &assets.hover_card,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(
                assets.hover_card.width() * scale_factor,
                assets.hover_card.height() * scale_factor,
            )),
            ..Default::default()
        },
    );
    let sprite = item.get_sprite();
    assets.items.draw_tile(
        x + 3.0 * scale_factor,
        y + 3.0 * scale_factor,
        sprite.x,
        sprite.y,
        Some(&DrawTextureParams {
            dest_size: Some(vec2(8.0 * scale_factor, 8.0 * scale_factor)),
            ..Default::default()
        }),
    );
    let name = item.get_name().to_uppercase();
    let long_name = name.len() > 10;
    draw_text_ex(
        &name,
        x + 13.0 * scale_factor,
        y + 10.0 * scale_factor - (if long_name { 4.0 * scale_factor } else { 0.0 }),
        TextParams {
            font: Some(&assets.font),
            font_size: (scale_factor * (if long_name { 4.0 } else { 8.0 })) as u16,
            ..Default::default()
        },
    );
    draw_multiline_text_ex(
        &item.get_desc(),
        x + 3.0 * scale_factor,
        y + 16.0 * scale_factor,
        None,
        TextParams {
            font: Some(&assets.font),
            font_size: (scale_factor * 4.0) as u16,
            ..Default::default()
        },
    );
}
pub fn draw_win_screen(mut win_time: f32, assets: &Assets) {
    let (actual_screen_width, actual_screen_height) = screen_size();
    let scale_factor = (actual_screen_width / SCREEN_WIDTH)
        .min(actual_screen_height / SCREEN_HEIGHT)
        .floor()
        .max(1.0);
    win_time = win_time.min(1.0);
    draw_rectangle(
        0.0,
        0.0,
        actual_screen_width,
        actual_screen_height * win_time,
        Color::from_hex(0x2ce8f5),
    );
    if win_time >= 1.0 {
        let x = (actual_screen_width - assets.win_screen.width() * scale_factor) / 2.0;
        let y = (actual_screen_height - assets.win_screen.height() * scale_factor) / 2.0;
        draw_texture_ex(
            &assets.win_screen,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    assets.win_screen.width() * scale_factor,
                    assets.win_screen.height() * scale_factor,
                )),
                ..Default::default()
            },
        );
        draw_text_ex(
            &"Thanks for playing!",
            x - 30.0 * scale_factor,
            (6.0 + 12.0) * scale_factor,
            TextParams {
                color: WHITE,
                font: Some(&assets.font),
                font_size: (scale_factor * 16.0) as u16,
                ..Default::default()
            },
        );
        draw_multiline_text_ex(
            &"VICTORY!\nYou won!",
            x + 22.0 * scale_factor,
            y + 6.0 * scale_factor + 29.0 * scale_factor,
            None,
            TextParams {
                color: WHITE,
                font: Some(&assets.font),
                font_size: (scale_factor * 8.0) as u16,
                ..Default::default()
            },
        );
    }
}
pub fn draw_dead_screen(
    mut dead_time: f32,
    assets: &Assets,
    player: &Player,
    floor: usize,
) -> bool {
    let (actual_screen_width, actual_screen_height) = screen_size();
    let scale_factor = (actual_screen_width / SCREEN_WIDTH)
        .min(actual_screen_height / SCREEN_HEIGHT)
        .floor()
        .max(1.0);
    let (mouse_x, mouse_y) = mouse_position();
    dead_time = dead_time.min(1.0);
    draw_rectangle(
        0.0,
        0.0,
        actual_screen_width,
        actual_screen_height * dead_time,
        BLACK,
    );
    if dead_time >= 1.0 {
        let x = (actual_screen_width - assets.gravestone.width() * scale_factor) / 2.0;
        let y = (actual_screen_height - assets.gravestone.height() * scale_factor) / 2.0;
        draw_texture_ex(
            &assets.gravestone,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    assets.gravestone.width() * scale_factor,
                    assets.gravestone.height() * scale_factor,
                )),
                ..Default::default()
            },
        );
        draw_multiline_text_ex(
            &format!(
                "Died at floor: {floor}\nEnemies slayn: {}",
                player.enemies_slayed
            ),
            x + 11.0 * scale_factor,
            y + 6.0 * scale_factor + 38.0 * scale_factor,
            None,
            TextParams {
                color: BLACK,
                font: Some(&assets.font),
                font_size: (scale_factor * 6.0) as u16,
                ..Default::default()
            },
        );
        let button_x = x + 25.0 * scale_factor;
        let button_y = y + 73.0 * scale_factor;
        let hovered = (button_x..button_x + 25.0 * scale_factor).contains(&mouse_x)
            && (button_y..button_y + 12.0 * scale_factor).contains(&mouse_y);

        let color = if hovered { GOLD } else { BLACK };

        draw_text_ex(
            "Restart",
            button_x,
            button_y + 6.0 * scale_factor,
            TextParams {
                color,
                font: Some(&assets.font),
                font_size: (scale_factor * 6.0) as u16,
                ..Default::default()
            },
        );

        is_mouse_button_pressed(MouseButton::Left) && hovered
    } else {
        false
    }
}
pub fn draw_ui(
    state: &mut InventoryState,
    player: &mut Player,
    assets: &Assets,
    dungeon: &Dungeon,
) {
    let (actual_screen_width, actual_screen_height) = screen_size();
    let scale_factor = (actual_screen_width / SCREEN_WIDTH)
        .min(actual_screen_height / SCREEN_HEIGHT)
        .floor()
        .max(1.0);
    let (mouse_x, mouse_y) = mouse_position();

    // draw healthbar

    draw_rectangle(
        (2.0 + 10.0) * scale_factor,
        (2.0 + 2.0) * scale_factor,
        84.0 * scale_factor,
        9.0 * scale_factor,
        BLACK,
    );
    draw_rectangle(
        (2.0 + 10.0) * scale_factor,
        (2.0 + 2.0) * scale_factor,
        (84.0 * scale_factor * player.health / MAX_PLAYER_HP).floor(),
        9.0 * scale_factor,
        Color::from_hex(0x63c74d),
    );

    draw_texture_ex(
        &assets.hp,
        2.0 * scale_factor,
        2.0 * scale_factor,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(
                assets.hp.width() * scale_factor,
                assets.hp.height() * scale_factor,
            )),
            ..Default::default()
        },
    );

    let clicking = is_mouse_button_pressed(MouseButton::Left);

    let x = (actual_screen_width - (assets.inventory.width() + 8.0) * scale_factor).floor();
    let y =
        ((actual_screen_height - (assets.inventory.height() + 11.0) * scale_factor) / 2.0).floor();

    let mut none_action = InventoryAction::None;
    match state {
        InventoryState::Inventory(action) => {
            let mut action = action;
            let hovered_index = player.inventory.iter().enumerate().position(|(i, _)| {
                let (slot_x, slot_y) = slot_index_position(i);

                ((slot_x as f32 * scale_factor + x)
                    ..(slot_x as f32 * scale_factor + x + 12.0 * scale_factor))
                    .contains(&mouse_x)
                    && ((slot_y as f32 * scale_factor + y)
                        ..(slot_y as f32 * scale_factor + y + 12.0 * scale_factor))
                        .contains(&mouse_y)
            });
            if clicking && let Some(i) = hovered_index {
                match &action {
                    InventoryAction::MovingItem(index) => {
                        if item_can_go_in_slot(&player.inventory[i], *index)
                            && item_can_go_in_slot(&player.inventory[*index], i)
                        {
                            (player.inventory[i], player.inventory[*index]) =
                                (player.inventory[*index], player.inventory[i]);
                            action = &mut none_action;
                            *state = InventoryState::Inventory(InventoryAction::None);
                        }
                    }
                    InventoryAction::CombiningItem(index, combinables) => {
                        if combinables.contains(&i) {
                            let new = combine(
                                player.inventory[*index].take().unwrap(),
                                player.inventory[i].take().unwrap(),
                            );
                            player.inventory[i] = Some(new);
                            action = &mut none_action;
                            *state = InventoryState::Inventory(InventoryAction::None);
                        }
                    }
                    InventoryAction::None => {
                        if player.inventory[i].is_some() {
                            action = &mut none_action;
                            *state = InventoryState::Inventory(InventoryAction::CtxMenuOpen(
                                i,
                                mouse_x - assets.ctx_menu.width() * scale_factor
                                    + 2.0 * scale_factor,
                                mouse_y - assets.ctx_menu.height() * scale_factor
                                    + 2.0 * scale_factor,
                            ));
                        }
                    }
                    _ => {}
                }
            }

            draw_texture_ex(
                &assets.inventory,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(
                        assets.inventory.width() * scale_factor,
                        assets.inventory.height() * scale_factor,
                    )),
                    ..Default::default()
                },
            );
            // draw player portrait
            let player_portrait = vec2(13.0, 23.0) * scale_factor;
            if let Some(Item::Weapon(item)) = &player.inventory[0] {
                assets.items.draw_tile(
                    x + player_portrait.x - 4.0 * scale_factor * 2.0,
                    y + player_portrait.y - 2.0 * scale_factor * 2.0,
                    item.sprite_x,
                    item.sprite_y,
                    Some(&DrawTextureParams {
                        dest_size: Some(vec2(16.0 * scale_factor, 16.0 * scale_factor)),
                        ..Default::default()
                    }),
                );
            }
            if let Some(Item::Armor(item)) = &player.inventory[1] {
                assets.items.draw_tile(
                    x + player_portrait.x,
                    y + player_portrait.y,
                    item.sprite_x + 1.0,
                    item.sprite_y,
                    Some(&DrawTextureParams {
                        dest_size: Some(vec2(16.0 * scale_factor, 16.0 * scale_factor)),
                        ..Default::default()
                    }),
                );
            }

            for (i, slot) in player.inventory.iter().enumerate() {
                let (slot_x, slot_y) = slot_index_position(i);
                let hovered = hovered_index.is_some_and(|f| f == i);

                let draw_x = slot_x as f32 * scale_factor + x + 2.0 * scale_factor;
                let draw_y = slot_y as f32 * scale_factor + y + 2.0 * scale_factor;
                if hovered || slot.is_some() || i >= 2 {
                    draw_rectangle(
                        draw_x,
                        draw_y,
                        8.0 * scale_factor,
                        8.0 * scale_factor,
                        if !hovered { UI_BORDER } else { UI_BACKGROUND },
                    );
                }
                if let InventoryAction::MovingItem(moving_index) = action
                    && i == *moving_index
                {
                    continue;
                }
                if let InventoryAction::CombiningItem(moving_index, _) = action
                    && i == *moving_index
                {
                    continue;
                }
                if let Some(item) = slot {
                    let sprite = item.get_sprite();
                    assets.items.draw_tile(
                        draw_x,
                        draw_y,
                        sprite.x,
                        sprite.y,
                        Some(&DrawTextureParams {
                            dest_size: Some(vec2(8.0 * scale_factor, 8.0 * scale_factor)),
                            ..Default::default()
                        }),
                    );
                }
                if slot.is_some()
                    && let InventoryAction::CombiningItem(_, combinables) = &action
                    && !combinables.contains(&i)
                {
                    draw_rectangle(
                        draw_x - 2.0 * scale_factor,
                        draw_y - 2.0 * scale_factor,
                        12.0 * scale_factor,
                        12.0 * scale_factor,
                        BLACK.with_alpha(0.5),
                    );
                }
            }

            let cursor_item = match &action {
                InventoryAction::MovingItem(index) => Some(index),
                InventoryAction::CombiningItem(index, _) => Some(index),
                _ => None,
            };
            if let Some(cursor_item) = cursor_item {
                let item = &player.inventory[*cursor_item].unwrap();
                let sprite = item.get_sprite();
                assets.items.draw_tile(
                    mouse_x - 4.0 * scale_factor,
                    mouse_y - 4.0 * scale_factor,
                    sprite.x,
                    sprite.y,
                    Some(&DrawTextureParams {
                        dest_size: Some(vec2(8.0 * scale_factor, 8.0 * scale_factor)),
                        ..Default::default()
                    }),
                );
            }

            if let Some(hover) = hovered_index
                && let Some(item) = &player.inventory[hover]
                && let InventoryAction::None = action
            {
                draw_item_hover_info(item, assets, mouse_x, mouse_y, scale_factor);
            }
            if let InventoryAction::CtxMenuOpen(item_index, mx, my) = action {
                let w = assets.ctx_menu.width() * scale_factor;
                let h = assets.ctx_menu.height() * scale_factor;
                draw_texture_ex(
                    &assets.ctx_menu,
                    *mx,
                    *my,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(w, h)),
                        ..Default::default()
                    },
                );
                let player_free_slot = player.get_free_slot();
                let player_first_free = player.inventory[0].is_none();
                let player_second_free = player.inventory[1].is_none();
                let item_index = *item_index;

                let combinable = get_combinable(&player.inventory, item_index);
                let has_combinable = !combinable.is_empty();

                let mut buttons: [CtxMenuButton; 5] = [
                    ("Move", &|_| true, &|state, _| {
                        *state = InventoryState::Inventory(InventoryAction::MovingItem(item_index))
                    }),
                    (
                        "Equip",
                        &|item| match item {
                            Item::Weapon(_) => {
                                if item_index == 0 {
                                    player_free_slot.is_some()
                                } else {
                                    player_first_free
                                }
                            }
                            Item::Armor(_) => {
                                if item_index == 1 {
                                    player_free_slot.is_some()
                                } else {
                                    player_second_free
                                }
                            }
                            _ => false,
                        },
                        &|_, player| {
                            let target_index = match player.inventory[item_index].unwrap() {
                                Item::Weapon(_) => 0,
                                Item::Armor(_) => 1,
                                _ => panic!(),
                            };
                            if item_index == target_index {
                                (
                                    player.inventory[item_index],
                                    player.inventory[player_free_slot.unwrap()],
                                ) = (
                                    player.inventory[player_free_slot.unwrap()],
                                    player.inventory[item_index],
                                );
                            } else {
                                (player.inventory[item_index], player.inventory[target_index]) =
                                    (player.inventory[target_index], player.inventory[item_index]);
                            }
                        },
                    ),
                    // todo: add these
                    ("Combine", &|_| has_combinable, &|state, _| {
                        *state = InventoryState::Inventory(InventoryAction::CombiningItem(
                            item_index,
                            combinable.clone(),
                        ));
                    }),
                    ("Throw", &|item| item.throwable().is_some(), &|state, _| {
                        *state = InventoryState::ThrowingItem(item_index)
                    }),
                    ("Drop", &|_| true, &|_, player| {
                        player.should_drop_item = Some(item_index);
                    }),
                ];
                let consume_button: CtxMenuButton = ("Consume", &|_| true, &|_, player| {
                    player.consume(item_index);
                });
                if let Item::Misc(item) = &player.inventory[item_index].unwrap()
                    && item.consumable.is_some()
                {
                    buttons[1] = consume_button;
                }
                let mut any_clicked = false;
                for (index, (mut text, cond, on_click)) in buttons.into_iter().enumerate() {
                    let x = *mx + 2.0 * scale_factor;
                    let y = *my + index as f32 * 7.0 * scale_factor + 2.0 * scale_factor;
                    let hovered = (x..(x + w)).contains(&mouse_x)
                        && (y..(y + 7.0 * scale_factor)).contains(&mouse_y);

                    let mut color = if hovered { GOLD } else { WHITE };
                    let disabled = !cond(&player.inventory[item_index].unwrap());
                    if disabled {
                        color = GRAY;
                    }
                    if !disabled && text == "Equip" && item_index <= 1 {
                        text = "Unequip";
                    }

                    draw_text_ex(
                        text,
                        x,
                        y + 6.0 * scale_factor,
                        TextParams {
                            color,
                            font: Some(&assets.font),
                            font_size: (scale_factor * 6.0) as u16,
                            ..Default::default()
                        },
                    );
                    if !disabled && hovered && clicking {
                        any_clicked = true;
                        *state = InventoryState::Inventory(InventoryAction::None);
                        on_click(state, player);
                        break;
                    }
                }
                if !any_clicked && clicking {
                    *state = InventoryState::Inventory(InventoryAction::None);
                }
            }
        }
        InventoryState::ThrowingItem(index) => {
            let item = &player.inventory[*index].unwrap();
            let sprite = item.get_sprite();
            assets.items.draw_tile(
                mouse_x - 4.0 * scale_factor,
                mouse_y - 4.0 * scale_factor,
                sprite.x,
                sprite.y,
                Some(&DrawTextureParams {
                    dest_size: Some(vec2(8.0 * scale_factor, 8.0 * scale_factor)),
                    ..Default::default()
                }),
            );
            if clicking {
                let scale_factor =
                    (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
                let (mouse_x, mouse_y) = (mouse_x / scale_factor, mouse_y / scale_factor);
                let mouse_tile = vec2(
                    (((mouse_x) / player.camera_zoom + player.camera_pos.x) / 8.0).floor(),
                    (((mouse_y) / player.camera_zoom + player.camera_pos.y) / 8.0).floor(),
                );

                player.should_throw_item = Some((*index, mouse_tile));
                *state = InventoryState::Closed;
            }
        }
        _ => match &dungeon.tiles[player.x + player.y * TILES_HORIZONTAL] {
            Tile::Chest(_, _, _) => {
                draw_tooltip("E: interact", assets);
            }
            Tile::Door => {
                draw_tooltip("E: descend", assets);
            }
            Tile::Ore(_, _, _) => {
                if player.has_pickaxe() {
                    draw_tooltip("E: Mine ore", assets);
                } else {
                    draw_tooltip("Pickaxe required", assets);
                }
            }
            _ => {
                if dungeon
                    .items
                    .iter()
                    .any(|(x, y, _)| *x == player.x && *y == player.y)
                {
                    draw_tooltip("E: pick up", assets);
                }
            }
        },
    }
}

fn item_can_go_in_slot(item: &Option<Item>, slot: usize) -> bool {
    match item {
        Some(item) => match item {
            Item::Armor(_) => slot >= 1,
            Item::Weapon(_) => slot == 0 || slot > 1,
            Item::Misc(_) => slot > 1,
        },
        None => true,
    }
}

type CtxMenuButton<'a> = (
    &'a str,
    &'a dyn Fn(&Item) -> bool,
    &'a dyn Fn(&mut InventoryState, &mut Player),
);
