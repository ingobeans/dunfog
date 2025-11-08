use macroquad::prelude::*;

use crate::{assets::Assets, entities::Player, utils::*};

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
        2..11 => {
            let index = index - 2;
            let x = index % 3;
            let y = index / 3;
            (inventory_start.0 + x * 13, inventory_start.1 + y * 13)
        }
        _ => {
            panic!("we don't have that many slots :(")
        }
    }
}
pub fn draw_inventory(
    state: &mut InventoryState,
    player: &mut Player,
    assets: &Assets,
    mouse_x: f32,
    mouse_y: f32,
) {
    let x = SCREEN_WIDTH - assets.inventory.width() - 8.0;
    let y = SCREEN_HEIGHT - assets.inventory.height() - 11.0;

    let clicking = is_mouse_button_pressed(MouseButton::Left);

    if clicking && (x..x + 15.0).contains(&mouse_x) && (y..y + 11.0).contains(&mouse_y) {
        *state = InventoryState::Inventory
    }
    if clicking
        && (x + 20.0..x + 20.0 + 15.0).contains(&mouse_x)
        && (y..y + 11.0).contains(&mouse_y)
    {
        *state = InventoryState::Crafting
    }

    match state {
        InventoryState::Inventory => {
            let hovered_index = player.inventory.iter().enumerate().position(|(i, _)| {
                let (slot_x, slot_y) = slot_index_position(i);

                let hovered = ((slot_x as f32 + x)..(slot_x as f32 + x + 12.0)).contains(&mouse_x)
                    && ((slot_y as f32 + y)..(slot_y as f32 + y + 12.0)).contains(&mouse_y);
                hovered
            });
            if clicking && let Some(i) = hovered_index {
                std::mem::swap(&mut player.inventory[i], &mut player.cursor_item);
            } else if clicking {
                // horse
            }

            draw_texture(&assets.inventory, x, y, WHITE);
            for (i, slot) in player.inventory.iter().enumerate() {
                let (slot_x, slot_y) = slot_index_position(i);
                let hovered = hovered_index.is_some_and(|f| f == i);

                if hovered || slot.is_some() || i >= 2 {
                    draw_rectangle(
                        slot_x as f32 + x + 2.0,
                        slot_y as f32 + y + 2.0,
                        8.0,
                        8.0,
                        if !hovered { UI_BORDER } else { UI_BACKGROUND },
                    );
                }
                if let Some(item) = slot {
                    let sprite = item.get_sprite();
                    assets.items.draw_tile(
                        slot_x as f32 + x + 2.0,
                        slot_y as f32 + y + 2.0,
                        sprite.x,
                        sprite.y,
                        None,
                    );
                }
            }

            if let Some(item) = &player.cursor_item {
                let sprite = item.get_sprite();
                assets
                    .items
                    .draw_tile(mouse_x - 4.0, mouse_y - 4.0, sprite.x, sprite.y, None);
            }
        }
        InventoryState::Crafting => {
            draw_texture(&assets.crafting, x, y, WHITE);
        }
        InventoryState::Closed => {}
    }
}
