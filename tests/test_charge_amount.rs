use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::calculate;
use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::constant::patches::attr::ATTR_CHARGE_AMOUNT;
use eve_fit_os::fit::{
    FitContainer, ItemCharge, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

const SHIP_SABRE: i32 = 22456;
const MODULE_INTERDICTION_SPHERE_LAUNCHER: i32 = 22782; // 15 m3 capacity
const CHARGE_WARP_DISRUPT_PROBE: i32 = 22778; // 5 m3 volume

fn skills() -> HashMap<i32, u8> {
    let rdr = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
    serde_json::from_reader(rdr).unwrap()
}

#[test]
fn test_interdiction_sphere_launcher_holds_three_probes() {
    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: Default::default(),
        ship_type_id: SHIP_SABRE,
        modules: vec![ItemModule {
            item_id: ItemID::Item(MODULE_INTERDICTION_SPHERE_LAUNCHER),
            slot: ItemSlot {
                slot_type: ItemSlotType::High,
                index: 0,
            },
            state: ItemState::Active,
            charge: Some(ItemCharge {
                type_id: CHARGE_WARP_DISRUPT_PROBE,
            }),
        }],
        drones: vec![],
        implants: vec![],
        boosters: vec![],
    };

    let container = FitContainer::new(fit, skills(), Default::default());
    let ship = calculate(&container, &info);

    let charge_amount = ship.modules[0].attributes[&ATTR_CHARGE_AMOUNT]
        .value
        .unwrap();
    assert_eq!(
        charge_amount, 3.0,
        "15 m3 launcher must hold 3 probes of 5 m3, got {charge_amount}"
    );
}
