use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::fit::{
    FitContainer, ItemFit, ItemImplant, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

#[test]
fn test_implant_group() {
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
            item_id: ItemID::Item(10842),
            slot: ItemSlot {
                slot_type: ItemSlotType::High,
                index: 1,
            },
            state: ItemState::Active,
            charge: None,
        }],
        drones: vec![],
        implants: vec![
            ItemImplant {
                type_id: 20121,
                index: 1,
            },
            ItemImplant {
                type_id: 20157,
                index: 2,
            },
            ItemImplant {
                type_id: 20158,
                index: 3,
            },
            ItemImplant {
                type_id: 20159,
                index: 4,
            },
            ItemImplant {
                type_id: 20160,
                index: 5,
            },
            ItemImplant {
                type_id: 20161,
                index: 6,
            },
        ],
        boosters: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2")).unwrap();

    let out = calculate(&container, &info);

    for implant in out.implants.iter() {
        println!("{:?}", implant.attributes.get(&838).unwrap().value);
        println!("{:?}", implant.attributes.get(&548).and_then(|u| u.value));
    }
    for module in out.modules.iter() {
        println!("{:?}", module.attributes.get(&548).and_then(|u| u.value));
    }
}
