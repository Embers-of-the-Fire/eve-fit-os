use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::constant::patches::attr::{
    ATTR_ARMOR_REMOTE_REPAIR_RATE, ATTR_HULL_REMOTE_REPAIR_RATE,
    ATTR_REMOTE_CAPACITOR_TRANSMITTER_RATE, ATTR_SHIELD_REMOTE_BOOST_RATE,
};
use eve_fit_os::fit::{
    FitContainer, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

#[test]
fn test_remote() {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 628,
        modules: vec![
            ItemModule {
                type_id: 4299,
                slot: ItemSlot {
                    slot_type: ItemSlotType::Medium,
                    index: 0,
                },
                state: ItemState::Active,
                charge: None,
            },
            ItemModule {
                type_id: 3588,
                slot: ItemSlot {
                    slot_type: ItemSlotType::Medium,
                    index: 0,
                },
                state: ItemState::Active,
                charge: None,
            },
            ItemModule {
                type_id: 26912,
                slot: ItemSlot {
                    slot_type: ItemSlotType::Medium,
                    index: 0,
                },
                state: ItemState::Active,
                charge: None,
            },
            ItemModule {
                type_id: 1190,
                slot: ItemSlot {
                    slot_type: ItemSlotType::Medium,
                    index: 0,
                },
                state: ItemState::Active,
                charge: None,
            },
        ],
        drones: vec![],
        implants: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5);

    let info =
        Database::init(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2")).unwrap();

    let out = calculate(&container, &info);

    let rep = out
        .hull
        .attributes
        .get(&ATTR_HULL_REMOTE_REPAIR_RATE)
        .and_then(|t| t.value)
        .unwrap_or_default();
    println!("remote hull repair damage: {:?}", rep);

    let rep = out
        .hull
        .attributes
        .get(&ATTR_SHIELD_REMOTE_BOOST_RATE)
        .and_then(|t| t.value)
        .unwrap_or_default();
    println!("remote shield repair damage: {:?}", rep);

    let rep = out
        .hull
        .attributes
        .get(&ATTR_ARMOR_REMOTE_REPAIR_RATE)
        .and_then(|t| t.value)
        .unwrap_or_default();
    println!("remote armor repair damage: {:?}", rep);

    let rep = out
        .hull
        .attributes
        .get(&ATTR_REMOTE_CAPACITOR_TRANSMITTER_RATE)
        .and_then(|t| t.value)
        .unwrap_or_default();
    println!("remote capacitor transmit: {:?}", rep);
}
