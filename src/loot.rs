use std::sync::LazyLock;

use crate::items::*;
use macroquad::prelude::*;

pub static BUSH_LOOT: LazyLock<LootTable> = LazyLock::new(|| {
    LootTable {
        //
        entries: vec![
            //
            (0.2, LootEntry::Item(Item::Misc(&STICK))),
        ],
    }
});

#[expect(dead_code)]
enum LootEntry {
    Item(Item),
    LootEntry(&'static LootTable),
}

fn weighted_choice(choices: &[(f32, LootEntry)]) -> &LootEntry {
    let mut total = 0.0;
    for entry in choices {
        total += entry.0;
    }
    let r = rand::rand();
    let r = r as f32 * total / u32::MAX as f32;
    let mut upto = 0.0;
    for (c, w) in choices {
        if upto + c >= r {
            return w;
        }
        upto += c
    }
    &choices[0].1
}

/// Specifies possible loot drops of an enemy / chest.
pub struct LootTable {
    entries: Vec<(f32, LootEntry)>,
}
impl LootTable {
    pub fn get_item(&self) -> &Item {
        let result = weighted_choice(&self.entries);
        match result {
            LootEntry::Item(item) => item,
            LootEntry::LootEntry(table) => table.get_item(),
        }
    }
}

#[cfg(test)]
mod tests {
    use macroquad::{
        miniquad,
        rand::{self},
    };

    use crate::{
        items::*,
        loot::{LootEntry, weighted_choice},
    };

    #[test]
    fn test_weighted_choice() {
        let seed = miniquad::date::now().to_bits();
        rand::srand(seed);
        let data = &[
            (0.5, LootEntry::Item(Item::Weapon(&DAGGER))),
            (0.5, LootEntry::Item(Item::Weapon(&BOW))),
            (0.5, LootEntry::Item(Item::Weapon(&MELEE))),
        ];
        weighted_choice(data);
    }
}
