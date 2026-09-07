use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::constant::patches::attr::ATTR_ARMOR_REPAIR_RATE;
use eve_fit_os::fit::{
    FitContainer, ItemCharge, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;
use eve_fit_os::provider::InfoProvider;

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
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let out = calculate(&container, &info);

    let rep = out
        .hull
        .attributes
        .get(&ATTR_ARMOR_REPAIR_RATE)
        .and_then(|t| t.value)
        .unwrap_or_default();
    println!("hull repair: {:?}", rep);
}

/// Regression test for <https://github.com/Embers-of-the-Fire/eve-fit-assistant/issues/452>.
///
/// Warfare buff `itemModifiers` describe ship-hull attributes (e.g. armor
/// resonances 267-270 for the resistance charge). They must not be injected
/// into module charges; otherwise every burst charge displays the resistance
/// charge's attributes whenever any burst module loads it.
#[test]
fn test_warfare_charge_attributes_not_polluted() {
    const ARMOR_RESONANCES: [i32; 4] = [267, 268, 269, 270];
    const ARMORED_COMMAND_BURST_II: i32 = 43552;
    // Armored command burst charges: resistance / repair amount / repair rate.
    const CHARGES: [i32; 3] = [42832, 42833, 42834];

    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 23919,
        modules: CHARGES
            .iter()
            .enumerate()
            .map(|(index, type_id)| ItemModule {
                item_id: ItemID::Item(ARMORED_COMMAND_BURST_II),
                slot: ItemSlot {
                    slot_type: ItemSlotType::High,
                    index: index as i32,
                },
                state: ItemState::Active,
                charge: Some(ItemCharge { type_id: *type_id }),
            })
            .collect(),
        drones: vec![],
        implants: vec![],
        boosters: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let out = calculate(&container, &info);

    // Charges must only carry their own dogma attributes; hull attributes
    // (armor resonances) must never leak into them.
    for (index, module) in out.modules.iter().enumerate() {
        let charge = module.charge.as_ref().expect("charge must be loaded");
        for attribute_id in ARMOR_RESONANCES {
            assert!(
                !charge.attributes.contains_key(&attribute_id),
                "charge of module {index} (type {}) must not contain hull attribute {attribute_id}",
                CHARGES[index],
            );
        }
    }

    // The buff itself must still reach the hull: the resistance charge's buff
    // changes the ship's armor EM resonance away from its base value.
    let base_em_resonance = info
        .get_dogma_attributes(23919)
        .iter()
        .find(|a| a.attribute_id == 267)
        .expect("ship must define armor EM resonance")
        .value;
    let hull_em_resonance = out
        .hull
        .attributes
        .get(&267)
        .and_then(|t| t.value)
        .unwrap_or_default();
    assert!(
        (hull_em_resonance - base_em_resonance).abs() > f64::EPSILON,
        "warfare buff must still modify hull resonance: base {base_em_resonance}, got {hull_em_resonance}",
    );
}
