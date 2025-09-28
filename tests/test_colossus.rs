use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::{ItemID, Object, SlotType};
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::fit::{
    FitContainer, ItemCharge, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

#[test]
#[ignore = "requires serenity-based data. Test only if you are targeting serenity."]
fn test_colossus() {
    let mut skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };
    skill_all_5.insert(78596, 5);
    skill_all_5.insert(78594, 1);

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 84925,
        modules: (0..3)
            .map(|index| ItemModule {
                item_id: ItemID::Item(37292),
                slot: ItemSlot {
                    slot_type: ItemSlotType::High,
                    index,
                },
                state: ItemState::Active,
                charge: Some(ItemCharge { type_id: 24521 }),
            })
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
        "DMG: {:#?}",
        out.modules
            .iter()
            .find(|m| m.slot.index == Some(0) && m.slot.slot_type == SlotType::High)
            .unwrap()
            .charge
            .as_ref()
            .unwrap()
            .attributes
            .get(&117)
    );

    println!(
        "SKILL: {:#?}",
        out.modules
            .iter()
            .find(|m| m.slot.index == Some(0) && m.slot.slot_type == SlotType::High)
            .unwrap()
            .charge
            .as_ref()
            .unwrap()
            .attributes
            .get(&117)
            .unwrap()
            .effects
            .iter()
            .filter_map(|e| match e.source {
                Object::Skill(id) => out.skills.get(id),
                _ => None,
            })
            .map(|t| t.item_id)
            .collect::<Vec<_>>()
    )
}
