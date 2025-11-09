use std::sync::LazyLock;

use crate::items::*;
use macroquad::prelude::*;

pub static BUSH_LOOT: LazyLock<LootTable> = LazyLock::new(|| {
    LootTable {
        //
        entries: vec![
            (1.0, LootEntry::Item(Item::Misc(&STICK))),
            (2.0, LootEntry::Item(Item::Misc(&LEAF))),
            (1.0, LootEntry::None),
        ],
    }
});
pub static MUSHROOM_LOOT: LazyLock<LootTable> = LazyLock::new(|| LootTable {
    entries: vec![(1.0, LootEntry::Item(Item::Misc(&POISON_MUSHROOM)))],
});
pub static SKELETON_DROPS: LazyLock<LootTable> = LazyLock::new(|| {
    LootTable {
        //
        entries: vec![
            (1.0, LootEntry::Item(Item::Misc(&STICK))),
            (1.0, LootEntry::Item(Item::Misc(&BONE))),
            (0.1, LootEntry::Item(Item::Weapon(&SHORTBOW))),
            (1.0, LootEntry::None),
        ],
    }
});
pub static ZOMBIE_DROPS: LazyLock<LootTable> = LazyLock::new(|| {
    LootTable {
        //
        entries: vec![
            (3.0, LootEntry::Item(Item::Misc(&FLESH))),
            (1.0, LootEntry::None),
        ],
    }
});

#[expect(dead_code)]
enum LootEntry {
    None,
    Item(Item),
    LootEntry(&'static LootTable),
}

fn weighted_choice(choices: &[(f32, LootEntry)]) -> &LootEntry {
    let mut total = 0.0;
    for entry in choices {
        total += entry.0;
    }
    let r = rand::gen_range(0.0, total);
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
    pub fn get_item(&self) -> Option<&Item> {
        let result = weighted_choice(&self.entries);
        match result {
            LootEntry::None => None,
            LootEntry::Item(item) => Some(item),
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
            (0.5, LootEntry::Item(Item::Weapon(&SHORTBOW))),
            (0.5, LootEntry::Item(Item::Weapon(&MELEE))),
        ];
        weighted_choice(data);
    }
}
