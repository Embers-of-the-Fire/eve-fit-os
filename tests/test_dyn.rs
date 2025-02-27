use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::{ItemID, Slot, SlotType};
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::constant::patches::attr::ATTR_SHIELD_BOOST_RATE;
use eve_fit_os::fit::{
    DynamicItem, FitContainer, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

#[test]
fn test_dyn() {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 603,
        modules: vec![ItemModule {
            item_id: ItemID::Dynamic(1),
            slot: ItemSlot {
                slot_type: ItemSlotType::Medium,
                index: 0,
            },
            state: ItemState::Active,
            charge: None,
        }],
        drones: vec![],
        implants: vec![],
    };

    let dyn_container = {
        let mut map = HashMap::new();
        map.insert(1, DynamicItem {
            base_type: 400,
            dynamic_attributes: [
                (73, 0.949999988079071),
                (50, 1.0),
                (68, 1.100000023841858),
                (6, 1.0),
                (30, 1.0),
            ]
            .into_iter()
            .collect(),
        });
        map
    };

    let container = FitContainer::new(fit, skill_all_5, dyn_container);

    let info =
        Database::init(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2")).unwrap();

    let out = calculate(&container, &info);

    let shield = out.hull.attributes.get(&ATTR_SHIELD_BOOST_RATE).unwrap();
    println!("Shield boost rate: {:?}", shield.value);

    let shield_item = out
        .modules
        .iter()
        .find(|t| {
            t.slot
                == Slot {
                    slot_type: SlotType::Medium,
                    index: Some(0),
                }
                && t.item_id == ItemID::Dynamic(1)
        })
        .unwrap()
        .attributes
        .get(&68);
    println!("Item shield boost rate: {:?}", shield_item);
}
