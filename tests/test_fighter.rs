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
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

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

#[test]
fn test_fighter_slot_index_is_group_id() {
    // Squadrons are expanded into one item per fighter, all sharing the
    // squadron's group id. The slot index must stay the group id (the app's
    // squadron index, as with drones) so consumers can locate a squadron's
    // calculated item; it must not be the position in the expanded list.
    let fit = ItemFit {
        damage_profile: DamageProfile::default(),
        ship_type_id: 23919,
        modules: vec![],
        drones: vec![],
        fighters: [
            (40556, 0u8, 3), // light fighter squadron, quantity 3
            (40560, 1u8, 2), // light fighter squadron, quantity 2
            (2948, 2u8, 1),  // heavy fighter squadron, quantity 1
        ]
        .into_iter()
        .flat_map(|(type_id, group_id, quantity)| {
            repeat_n(
                ItemFighter {
                    type_id,
                    group_id,
                    ability: FighterAbility::ATTACK_MISSILE | FighterAbility::MISSILES,
                },
                quantity,
            )
        })
        .collect(),
        implants: vec![],
        boosters: vec![],
    };

    let container = FitContainer::new(fit, Default::default(), Default::default());

    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let out = calculate(&container, &info);

    let fighters: Vec<_> = out
        .modules
        .iter()
        .filter(|item| matches!(item.slot.slot_type, SlotType::Fighter { .. }))
        .collect();
    assert_eq!(fighters.len(), 6);
    for item in fighters {
        let SlotType::Fighter { group_id, .. } = item.slot.slot_type else {
            unreachable!()
        };
        assert_eq!(
            item.slot.index,
            Some(group_id as i32),
            "fighter slot index must be the group id, not the expanded list position",
        );
    }
}
