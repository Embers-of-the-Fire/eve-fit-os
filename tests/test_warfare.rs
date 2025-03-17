use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::constant::patches::attr::ATTR_ARMOR_REPAIR_RATE;
use eve_fit_os::fit::{
    FitContainer, ItemCharge, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

#[test]
fn test_warfare() {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 23919,
        modules: vec![
            ItemModule {
                item_id: ItemID::Item(43552),
                slot: ItemSlot {
                    slot_type: ItemSlotType::High,
                    index: 0,
                },
                // state: ItemState::Active,
                state: ItemState::Active,
                charge: Some(ItemCharge { type_id: 42833 }),
            },
            ItemModule {
                item_id: ItemID::Item(3530),
                slot: ItemSlot {
                    slot_type: ItemSlotType::Low,
                    index: 0,
                },
                state: ItemState::Active,
                charge: None,
            },
        ],
        drones: vec![],
        implants: vec![],
        boosters: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2")).unwrap();

    let out = calculate(&container, &info);

    let rep = out
        .hull
        .attributes
        .get(&ATTR_ARMOR_REPAIR_RATE)
        .and_then(|t| t.value)
        .unwrap_or_default();
    println!("hull repair: {:?}", rep);
}
