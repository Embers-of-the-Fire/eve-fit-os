use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::constant::patches::attr::{
    ATTR_CAPACITOR_DEPLETES_IN, ATTR_CAPACITOR_PEAK_LOAD, ATTR_CAPACITOR_PEAK_RECHARGE,
    ATTR_SHIELD_BOOST_RATE,
};
use eve_fit_os::fit::{
    FitContainer, ItemCharge, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

#[test]
fn test_cap() {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 628,
        modules: vec![ItemModule {
            item_id: ItemID::Item(32772),
            slot: ItemSlot {
                slot_type: ItemSlotType::Medium,
                index: 0,
            },
            state: ItemState::Active,
            charge: Some(ItemCharge { type_id: 3554 }),
            // charge: None,
        }],
        drones: vec![],
        implants: vec![],
        boosters: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2")).unwrap();

    let out = calculate(&container, &info);

    println!(
        "cap: {:?}",
        out.hull
            .attributes
            .get(&ATTR_CAPACITOR_PEAK_RECHARGE)
            .unwrap()
            .value
            .unwrap_or_default()
            - out
                .hull
                .attributes
                .get(&ATTR_CAPACITOR_PEAK_LOAD)
                .unwrap()
                .value
                .unwrap_or_default(),
    );

    println!(
        "item cap: {:?}, charge cap: {:?}",
        out.modules
            .iter()
            .find(|u| u.item_id == ItemID::Item(32772))
            .unwrap()
            .attributes
            // .get(&ATTR_CAPACITOR_PEAK_LOAD_WITH_BOOST)
            .get(&ATTR_CAPACITOR_PEAK_LOAD)
            .unwrap()
            .value,
        out.modules
            .iter()
            .find(|u| u.item_id == ItemID::Item(32772))
            .unwrap()
            .charge
            .as_ref()
            .unwrap()
            .attributes
            .get(&67)
            .unwrap()
            .value,
    );

    println!(
        "shield repair: {:?}",
        out.hull
            .attributes
            .get(&ATTR_SHIELD_BOOST_RATE)
            .and_then(|u| u.value)
    );
}

#[test]
fn test_cap_raw() {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 47271,
        modules: (0..4)
            .map(|t| ItemModule {
                item_id: ItemID::Item(15154),
                slot: ItemSlot {
                    slot_type: ItemSlotType::Medium,
                    index: t,
                },
                state: ItemState::Active,
                charge: None,
            })
            .chain(std::iter::once(ItemModule {
                item_id: ItemID::Item(15154),
                slot: ItemSlot {
                    slot_type: ItemSlotType::Medium,
                    index: 4,
                },
                state: ItemState::Active,
                charge: None,
            }))
            .collect(),
        drones: vec![],
        implants: vec![],
        boosters: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2")).unwrap();

    let out = calculate(&container, &info);

    println!(
        "cap: {:?}",
        out.hull
            .attributes
            .get(&ATTR_CAPACITOR_PEAK_RECHARGE)
            .unwrap()
            .value
            .unwrap_or_default()
            - out
                .hull
                .attributes
                .get(&ATTR_CAPACITOR_PEAK_LOAD)
                .unwrap()
                .value
                .unwrap_or_default(),
    );

    println!(
        "depletes in: {:?}",
        out.hull
            .attributes
            .get(&ATTR_CAPACITOR_DEPLETES_IN)
            .unwrap()
            .value
            .unwrap_or_default()
    );

    println!(
        "item cap: {:?}",
        out.modules
            .iter()
            .find(|u| u.item_id == ItemID::Item(15154))
            .unwrap()
            .attributes
            // .get(&ATTR_CAPACITOR_PEAK_LOAD_WITH_BOOST)
            .get(&ATTR_CAPACITOR_PEAK_LOAD)
            .unwrap()
            .value,
    );
}
