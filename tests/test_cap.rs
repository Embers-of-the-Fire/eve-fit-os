use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::constant::patches::attr::{
    ATTR_CAPACITOR_PEAK_LOAD, ATTR_CAPACITOR_PEAK_RECHARGE, ATTR_SHIELD_BOOST_RATE,
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
            type_id: 32772,
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
    };

    let container = FitContainer::new(fit, skill_all_5);

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
            .find(|u| u.type_id == 32772)
            .unwrap()
            .attributes
            // .get(&ATTR_CAPACITOR_PEAK_LOAD_WITH_BOOST)
            .get(&ATTR_CAPACITOR_PEAK_LOAD)
            .unwrap()
            .value,
        out.modules
            .iter()
            .find(|u| u.type_id == 32772)
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
