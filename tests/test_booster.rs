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
        Database::init(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2")).unwrap();

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
