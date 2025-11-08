use macroquad::{miniquad::window::screen_size, prelude::*};

use crate::{
    assets::Assets,
    entities::{Item, Player},
    utils::*,
};

const UI_BACKGROUND: Color = Color::from_hex(0x8b9bb4);
const UI_BORDER: Color = Color::from_hex(0xc0cbdc);

pub enum InventoryState {
    Closed,
    Inventory,
    Crafting,
}
impl InventoryState {
    pub fn toggle(&mut self) {
        *self = match self {
            InventoryState::Closed => InventoryState::Inventory,
            InventoryState::Inventory => InventoryState::Closed,
            InventoryState::Crafting => InventoryState::Closed,
        }
    }
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
pub fn draw_inventory(state: &mut InventoryState, player: &mut Player, assets: &Assets) {
    let (actual_screen_width, actual_screen_height) = screen_size();
    let scale_factor = (actual_screen_width / SCREEN_WIDTH)
        .min(actual_screen_height / SCREEN_HEIGHT)
        .floor()
        .max(1.0);
    let (mouse_x, mouse_y) = mouse_position();

    let x = (actual_screen_width - (assets.inventory.width() + 8.0) * scale_factor).floor();
    let y =
        ((actual_screen_height - (assets.inventory.height() + 11.0) * scale_factor) / 2.0).floor();

    let clicking = is_mouse_button_pressed(MouseButton::Left);

    if clicking
        && (x..x + 15.0 * scale_factor).contains(&mouse_x)
        && (y..y + 11.0 * scale_factor).contains(&mouse_y)
    {
        *state = InventoryState::Inventory
    }
    if clicking
        && (x + 20.0 * scale_factor..x + (20.0 + 15.0) * scale_factor).contains(&mouse_x)
        && (y..y + 11.0 * scale_factor).contains(&mouse_y)
    {
        *state = InventoryState::Crafting
    }

    match state {
        InventoryState::Inventory => {
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
                std::mem::swap(&mut player.inventory[i], &mut player.cursor_item);
            } else if clicking {
                // close inventory
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
            }

            if let Some(item) = &player.cursor_item {
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
        }
        InventoryState::Crafting => {
            draw_texture_ex(
                &assets.crafting,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(
                        assets.crafting.width() * scale_factor,
                        assets.crafting.height() * scale_factor,
                    )),
                    ..Default::default()
                },
            );
        }
        InventoryState::Closed => {}
    }
}
