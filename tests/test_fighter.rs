use std::collections::HashMap;
use std::fs::File;
use std::iter::repeat_n;

use eve_fit_os::calculate::item::{FighterAbility, SlotType};
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::constant::patches::attr::{
    ATTR_FIGHTER_DAMAGE_PER_SECOND, ATTR_FIGHTER_MISSILES_DAMAGE_PER_SECOND,
};
use eve_fit_os::fit::{FitContainer, ItemFighter, ItemFit};
use eve_fit_os::protobuf::Database;

#[test]
fn test_fighter() {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        damage_profile: DamageProfile::default(),
        ship_type_id: 23919,
        modules: vec![],
        drones: vec![],
        fighters: repeat_n(
            ItemFighter {
                type_id: 40556,
                group_id: 0,
                ability: FighterAbility::ATTACK_MISSILE | FighterAbility::MISSILES,
            },
            9,
        )
        .chain(repeat_n(
            ItemFighter {
                type_id: 40560,
                group_id: 0,
                ability: FighterAbility::ATTACK_MISSILE | FighterAbility::MISSILES,
            },
            6,
        ))
        .collect(),
        implants: vec![],
        boosters: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2")).unwrap();

    let out = calculate(&container, &info);

    let fighter_attack_dps = out
        .modules
        .iter()
        .find(|t| {
            matches!(
                t.slot.slot_type,
                SlotType::Fighter {
                    group_id: n,
                    ..
                } if n == 0
            )
        })
        .expect("fighter not found")
        .attributes
        .get(&ATTR_FIGHTER_MISSILES_DAMAGE_PER_SECOND)
        // .get(&2226)
        .expect("No attribute")
        .value
        .expect("No value");
    println!("Fighter attack dps: {}", fighter_attack_dps);

    let ship_fighter_dps = out.hull
        .attributes
        .get(&ATTR_FIGHTER_DAMAGE_PER_SECOND)
        // .get(&2226)
        .expect("No attribute")
        .value
        .expect("No value");
    println!("Fighter sum attack dps: {}", ship_fighter_dps);
}
