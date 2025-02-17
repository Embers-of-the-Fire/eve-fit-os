use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::calculate;
use eve_fit_os::constant::patches::attr::ATTR_SHIELD_EFFECTIVE_BOOST_RATE;
use eve_fit_os::fit::{
    FitContainer, ItemCharge, ItemDrone, ItemFit, ItemImplant, ItemModule, ItemSlot,
    ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

#[test]
fn test_basic_fit() {
    let skill_all_5: HashMap<i32, u8> = {
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
            .chain(vec![ItemModule {
                type_id: 10850,
                slot: ItemSlot {
                    slot_type: ItemSlotType::Medium,
                    index: 0,
                },
                state: ItemState::Overload,
                charge: None,
            }])
            .collect(),
        drones: vec![ItemDrone {
            type_id: 2456,
            state: ItemState::Active,
        }],
        implants: vec![
            ItemImplant {
                type_id: 57123,
                index: 1,
            },
            ItemImplant {
                type_id: 57124,
                index: 2,
            },
        ],
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
    println!("Missile damage: {:?}", raw_dmg);

    let shield = out
        .hull
        .attributes
        .get(&ATTR_SHIELD_EFFECTIVE_BOOST_RATE)
        .unwrap();
    println!("Shield boost rate: {:?}", shield.value);

    let el = out
        .implants
        .iter()
        .find(|t| t.type_id == 57123)
        .and_then(|t| t.attributes.get(&314))
        .and_then(|t| t.value);

    println!("Implant 57123 sub effect: {:?}", el);
}
