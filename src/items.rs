use crate::utils::*;
use macroquad::prelude::*;

pub static ITEM_COMBINATIONS: &[([Item; 2], Item)] = &[
    (
        [Item::Misc(&STONE), Item::Misc(&STICK)],
        Item::Weapon(&STONE_SPEAR),
    ),
    ([Item::Misc(&LEAF), Item::Misc(&LEAF)], Item::Misc(&FIBER)),
    (
        [Item::Misc(&FIBER), Item::Misc(&FIBER)],
        Item::Armor(&TUNIC),
    ),
];

pub fn combine(a: Item, b: Item) -> Item {
    for (combination, result) in ITEM_COMBINATIONS {
        if (combination[0] == a && combination[1] == b)
            || combination[1] == a && combination[0] == b
        {
            return result.clone();
        }
    }
    panic!("no combination for these items exist!")
}

pub fn get_combinable(items: &[Option<Item>], index: usize) -> Vec<usize> {
    if items[index].is_none() {
        return Vec::new();
    }
    let mut combinable = Vec::new();
    for (i, item) in items.iter().enumerate() {
        if i == index {
            continue;
        }
        let Some(item) = item else {
            continue;
        };
        for (combination, _) in ITEM_COMBINATIONS {
            if (&combination[0] == item && &combination[1] == &items[index].unwrap())
                || &combination[1] == item && &combination[0] == &items[index].unwrap()
            {
                combinable.push(i);
            }
        }
    }
    combinable
}

#[derive(Clone, PartialEq)]
pub struct Armor {
    pub block_chance: f32,
    pub sprite_x: f32,
    pub sprite_y: f32,
    pub name: &'static str,
}
impl Armor {
    fn get_desc(&self) -> String {
        format!("Block Chance: {}", self.block_chance)
    }
}
#[derive(Clone, PartialEq)]
pub struct Weapon {
    pub attack_range: std::ops::Range<usize>,
    pub base_damage: f32,
    pub sprite_x: f32,
    pub sprite_y: f32,
    pub name: &'static str,
    pub fires_particle: Option<(f32, f32)>,
    pub throwable: Option<(f32, Vec2)>,
}
impl Weapon {
    fn get_desc(&self) -> String {
        format!(
            "DMG: {}\nRANGE: {}",
            self.base_damage,
            serialize_range(&self.attack_range)
        )
    }
}
pub const MELEE: Weapon = Weapon {
    attack_range: 1..2,
    base_damage: 1.0,
    sprite_x: 0.0,
    sprite_y: 0.0,
    name: "melee",
    fires_particle: None,
    throwable: None,
};
pub const DAGGER: Weapon = Weapon {
    attack_range: 1..2,
    base_damage: 2.0,
    sprite_x: 1.0,
    sprite_y: 0.0,
    name: "dagger",
    fires_particle: None,
    throwable: None,
};
pub const BOW: Weapon = Weapon {
    attack_range: 2..4,
    base_damage: 2.0,
    sprite_x: 2.0,
    sprite_y: 0.0,
    name: "bow",
    fires_particle: Some((0.0, 0.0)),
    throwable: None,
};
pub const SPELLBOOK: Weapon = Weapon {
    attack_range: 2..4,
    base_damage: 5.0,
    sprite_x: 3.0,
    sprite_y: 0.0,
    name: "spellbook",
    fires_particle: Some((3.0, 0.0)),
    throwable: None,
};
pub const STONE_SPEAR: Weapon = Weapon {
    attack_range: 1..2,
    sprite_x: 4.0,
    sprite_y: 0.0,
    base_damage: 2.0,
    fires_particle: None,
    name: "stone spear",
    throwable: Some((5.0, vec2(1.0, 0.0))),
};
pub const IRON_ARMOR: Armor = Armor {
    block_chance: 0.4,
    sprite_x: 0.0,
    sprite_y: 1.0,
    name: "iron armor",
};
pub const TUNIC: Armor = Armor {
    block_chance: 0.3,
    sprite_x: 2.0,
    sprite_y: 1.0,
    name: "leather tunic",
};
#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub enum StatusEffect {
    Poison,
}
#[derive(Clone, PartialEq)]
pub struct MiscItem {
    sprite_x: f32,
    sprite_y: f32,
    name: &'static str,
    desc: &'static str,
    throwable: Option<(f32, Vec2)>,
    pub consumable: Option<(f32, Option<StatusEffect>)>,
}
pub const STICK: MiscItem = MiscItem {
    sprite_x: 0.0,
    sprite_y: 2.0,
    name: "stick",
    desc: "a cool stick",
    throwable: None,
    consumable: None,
};
pub const BONE: MiscItem = MiscItem {
    sprite_x: 3.0,
    sprite_y: 2.0,
    name: "bone",
    desc: "a real bone",
    throwable: None,
    consumable: None,
};
pub const STONE: MiscItem = MiscItem {
    sprite_x: 1.0,
    sprite_y: 2.0,
    name: "stone",
    desc: "a small stone",
    throwable: Some((3.0, vec2(2.0, 0.0))),
    consumable: None,
};
pub const FLESH: MiscItem = MiscItem {
    sprite_x: 2.0,
    sprite_y: 2.0,
    name: "flesh",
    desc: "consumable flesh",
    throwable: None,
    consumable: Some((2.0, None)),
};
pub const LEAF: MiscItem = MiscItem {
    sprite_x: 4.0,
    sprite_y: 2.0,
    name: "leaf",
    desc: "can be crafted into fiber",
    throwable: None,
    consumable: None,
};
pub const FIBER: MiscItem = MiscItem {
    sprite_x: 5.0,
    sprite_y: 2.0,
    name: "fiber",
    desc: "useful for crafting clothes",
    throwable: None,
    consumable: None,
};
pub const POISON_MUSHROOM: MiscItem = MiscItem {
    sprite_x: 6.0,
    sprite_y: 2.0,
    name: "poisonous mushroom",
    desc: "maybe throw on your foes?",
    throwable: Some((0.0, vec2(4.0, 0.0))),
    consumable: Some((0.0, Some(StatusEffect::Poison))),
};
#[derive(Clone, Copy, PartialEq)]
pub enum Item {
    Weapon(&'static Weapon),
    Armor(&'static Armor),
    Misc(&'static MiscItem),
}
impl Item {
    pub fn get_sprite(&self) -> Vec2 {
        match &self {
            Item::Weapon(weapon) => vec2(weapon.sprite_x, weapon.sprite_y),
            Item::Armor(armor) => vec2(armor.sprite_x, armor.sprite_y),
            Item::Misc(misc_item) => vec2(misc_item.sprite_x, misc_item.sprite_y),
        }
    }
    pub fn get_name(&self) -> &'static str {
        match &self {
            Item::Weapon(weapon) => weapon.name,
            Item::Armor(armor) => armor.name,
            Item::Misc(misc_item) => misc_item.name,
        }
    }
    pub fn get_desc(&self) -> String {
        match &self {
            Item::Weapon(weapon) => weapon.get_desc(),
            Item::Armor(armor) => armor.get_desc(),
            Item::Misc(misc_item) => misc_item.desc.to_string(),
        }
    }
    pub fn throwable(&self) -> Option<(f32, Vec2)> {
        match &self {
            Item::Weapon(weapon) => weapon.throwable,
            Item::Armor(_) => None,
            Item::Misc(misc_item) => misc_item.throwable,
        }
    }
}
