use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::constant::patches::attr::ATTR_CAPACITOR_PEAK_DELTA;
use eve_fit_os::fit::{
    FitContainer, ItemBooster, ItemCharge, ItemFit, ItemModule, ItemSlot, ItemSlotType,
    ItemState,
};
use eve_fit_os::protobuf::Database;
use eve_fit_os::provider::InfoProvider;

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
            item_id: ItemID::Item(499),
            slot: ItemSlot {
                slot_type: ItemSlotType::High,
                index: 0,
            },
            state: ItemState::Active,
            charge: Some(ItemCharge { type_id: 210 }),
            // charge: None,
        }],
        drones: vec![],
        implants: vec![],
        boosters: vec![ItemBooster {
            type_id: 81083,
            index: 0,
        }],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let out = calculate(&container, &info);

    println!(
        "cap: {:?}",
        out.hull
            .attributes
            .get(&ATTR_CAPACITOR_PEAK_DELTA)
            .unwrap()
            .value
            .unwrap_or_default()
    );

    println!(
        "attr: {:?}",
        out.boosters
            .first()
            .unwrap()
            .attributes
            .iter()
            .map(|e| (e.0, e.1.value))
            .collect::<HashMap<_, _>>()
    );
}

/// Regression test for the booster cache reading from the charge cache:
/// a charge at the same index sharing dogma attribute IDs with the booster
/// must not pollute (or swallow) the booster's attribute values.
#[test]
fn test_booster_cache_not_polluted_by_charge() {
    const BOOSTER_TYPE_ID: i32 = 15466;
    const SHARED_ATTRIBUTES: [i32; 2] = [182, 277];

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
            // Armored Command Burst II
            item_id: ItemID::Item(43552),
            slot: ItemSlot {
                slot_type: ItemSlotType::High,
                index: 0,
            },
            state: ItemState::Active,
            charge: Some(ItemCharge { type_id: 42832 }),
        }],
        drones: vec![],
        implants: vec![],
        boosters: vec![ItemBooster {
            type_id: BOOSTER_TYPE_ID,
            index: 0,
        }],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let out = calculate(&container, &info);

    let booster_attributes = info.get_dogma_attributes(BOOSTER_TYPE_ID);
    for attribute_id in SHARED_ATTRIBUTES {
        let base = booster_attributes
            .iter()
            .find(|attr| attr.attribute_id == attribute_id)
            .unwrap()
            .value;
        assert_eq!(
            out.boosters[0].attributes[&attribute_id].value,
            Some(base),
            "booster attribute {attribute_id} must resolve to its own dogma value"
        );
    }
}
