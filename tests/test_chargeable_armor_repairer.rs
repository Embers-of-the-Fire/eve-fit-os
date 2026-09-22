use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::calculate;
use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::fit::{
    FitContainer, ItemCharge, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

const SHIP_VEXOR_NAVY_ISSUE: i32 = 17843; // +7.5%/level armorDamageAmount (Gallente Cruiser)
const MODULE_MEDIUM_ARMOR_REPAIRER_II: i32 = 3530;
const MODULE_MEDIUM_ANCILLARY_ARMOR_REPAIRER: i32 = 33101;
const CHARGE_NANITE_REPAIR_PASTE: i32 = 28668;
const RIG_MEDIUM_NANOBOT_ACCELERATOR_I: i32 = 31065; // -15% duration
const RIG_MEDIUM_AUXILIARY_NANO_PUMP_I: i32 = 31047; // +15% armorDamageAmount
const RIG_MEDIUM_TRIMARK_ARMOR_PUMP_I: i32 = 31055; // armor HP only

fn skills() -> HashMap<i32, u8> {
    let rdr = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
    serde_json::from_reader(rdr).unwrap()
}

fn module(
    item_id: i32,
    slot_type: ItemSlotType,
    index: i32,
    charge: Option<i32>,
) -> ItemModule {
    ItemModule {
        item_id: ItemID::Item(item_id),
        slot: ItemSlot { slot_type, index },
        state: ItemState::Active,
        charge: charge.map(|type_id| ItemCharge { type_id }),
        damage_turns: 0,
    }
}

fn fit(charge: Option<i32>) -> ItemFit {
    ItemFit {
        fighters: vec![],
        damage_profile: Default::default(),
        ship_type_id: SHIP_VEXOR_NAVY_ISSUE,
        modules: vec![
            module(
                MODULE_MEDIUM_ARMOR_REPAIRER_II,
                ItemSlotType::Medium,
                0,
                None,
            ),
            module(
                MODULE_MEDIUM_ANCILLARY_ARMOR_REPAIRER,
                ItemSlotType::Medium,
                1,
                charge,
            ),
            module(RIG_MEDIUM_NANOBOT_ACCELERATOR_I, ItemSlotType::Rig, 0, None),
            module(RIG_MEDIUM_AUXILIARY_NANO_PUMP_I, ItemSlotType::Rig, 1, None),
            module(RIG_MEDIUM_TRIMARK_ARMOR_PUMP_I, ItemSlotType::Rig, 2, None),
        ],
        drones: vec![],
        implants: vec![],
        boosters: vec![],
        system_buffs: vec![],
    }
}

fn attribute_value(
    ship: &eve_fit_os::calculate::Ship,
    module_index: usize,
    attribute_id: i32,
) -> f64 {
    ship.modules[module_index].attributes[&attribute_id]
        .value
        .unwrap()
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-6,
        "expected {expected}, got {actual}"
    );
}

/// Issue #485: the paste multiplier must not drop the effects collected on
/// armorDamageAmount (ship bonus, rigs) of an ancillary armor repairer.
#[test]
fn test_aar_with_paste_keeps_amount_modifiers() {
    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let container = FitContainer::new(
        fit(Some(CHARGE_NANITE_REPAIR_PASTE)),
        skills(),
        Default::default(),
    );
    let ship = calculate(&container, &info);

    // MAR II: 368 * 1.375 (ship) * 1.15 (rig); duration 12000 * 0.75 (skill) * 0.85 (rig).
    assert_close(attribute_value(&ship, 0, 84), 581.9);
    assert_close(attribute_value(&ship, 0, 73), 7650.0);

    // MAAR: 207 * 3 (paste) * 1.375 (ship) * 1.15 (rig); duration as above.
    assert_close(attribute_value(&ship, 1, 84), 981.95625);
    assert_close(attribute_value(&ship, 1, 73), 7650.0);
}

/// Without paste the multiplier must not apply, but the other modifiers must.
#[test]
fn test_aar_without_paste_keeps_amount_modifiers() {
    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let container = FitContainer::new(fit(None), skills(), Default::default());
    let ship = calculate(&container, &info);

    // MAAR: 207 * 1.375 (ship) * 1.15 (rig); duration as above.
    assert_close(attribute_value(&ship, 1, 84), 327.31875);
    assert_close(attribute_value(&ship, 1, 73), 7650.0);
}
