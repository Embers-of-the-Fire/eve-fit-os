use item::Item;

use crate::constant::CHARACTER_TYPE_ID;
use crate::provider::{FitProvider, InfoProvider};

pub mod item;
mod pass_1;
mod pass_2;
mod pass_3;
mod pass_4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageProfile {
    pub em: f64,
    pub explosive: f64,
    pub kinetic: f64,
    pub thermal: f64,
}

impl Default for DamageProfile {
    fn default() -> Self {
        Self {
            em: 0.25,
            explosive: 0.25,
            kinetic: 0.25,
            thermal: 0.25,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ship {
    pub hull: Item,
    pub modules: Vec<Item>,
    pub skills: Vec<Item>,
    pub implants: Vec<Item>,
    pub character: Item,
    pub damage_profile: DamageProfile,

    // not implemented yet
    pub structure: Item,
    pub target: Item,
}

impl Ship {
    pub fn new(ship_id: i32) -> Self {
        Self {
            hull: Item::new_fake(ship_id),
            modules: Vec::new(),
            skills: Vec::new(),
            implants: Vec::new(),
            character: Item::new_fake(CHARACTER_TYPE_ID),
            damage_profile: DamageProfile::default(),

            structure: Item::new_fake(0),
            target: Item::new_fake(0),
        }
    }

    pub fn new_with_damage_profile(
        ship_id: i32,
        damage_profile: DamageProfile,
    ) -> Self {
        Self {
            hull: Item::new_fake(ship_id),
            modules: Vec::new(),
            skills: Vec::new(),
            implants: Vec::new(),
            character: Item::new_fake(CHARACTER_TYPE_ID),
            damage_profile,
            structure: Item::new_fake(0),
            target: Item::new_fake(0),
        }
    }
}

pub fn calculate(fit: &impl FitProvider, info: &impl InfoProvider) -> Ship {
    let mut ship = Ship::new(fit.fit().ship_type_id);

    pass_1::pass(fit, info, &mut ship);
    pass_2::pass(fit, info, &mut ship);
    pass_3::pass(fit, info, &mut ship);
    pass_4::pass(fit, info, &mut ship);
    ship
}
