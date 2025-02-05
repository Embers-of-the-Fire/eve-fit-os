use std::collections::BTreeMap;
use std::fs::File;

use eve_fit_os::calculate::calculate;
use eve_fit_os::fit::{
    FitContainer, ItemCharge, ItemDrone, ItemFit, ItemImplant, ItemModule, ItemSlot,
    ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

#[test]
fn test_basic_fit() {
    let skill_all_5: BTreeMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        ship_type_id: 628,
        modules: (0..3)
            .map(|index| ItemModule {
                type_id: 1877,
                slot: ItemSlot {
                    slot_type: ItemSlotType::High,
                    index,
                },
                state: ItemState::Active,
                charge: Some(ItemCharge { type_id: 2613 }),
            })
            .collect(),
        drones: vec![ItemDrone {
            type_id: 2456,
            state: ItemState::Active,
        }],
        implants: vec![ItemImplant { type_id: 27252 }],
    };

    let container = FitContainer::new(fit, skill_all_5);

    let info =
        Database::init(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2")).unwrap();

    let out = calculate(&container, &info);

    let raw_dmg = out
        .modules
        .iter()
        .find(|t| t.type_id == 1877)
        .and_then(|t| t.charge.as_ref())
        .and_then(|t| t.attributes.get(&114))
        .and_then(|t| t.value)
        .unwrap_or_default();
    println!("Missile damage: {:?}", raw_dmg,)
}
