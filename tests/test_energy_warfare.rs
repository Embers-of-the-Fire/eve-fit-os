use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::{Item, ItemID};
use eve_fit_os::calculate::{DamageProfile, Ship, calculate};
use eve_fit_os::constant::patches::attr::{
    ATTR_ENERGY_NEUTRALIZER_RATE, ATTR_ENERGY_NOSFERATU_RATE,
};
use eve_fit_os::fit::{
    FitContainer, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

const SHIP_ARBITRATOR: i32 = 628;
const MODULE_MEDIUM_ENERGY_NEUTRALIZER_II: i32 = 12267;
const MODULE_MEDIUM_ENERGY_NOSFERATU_II: i32 = 12259;
const MODULE_SMALL_REMOTE_CAPACITOR_TRANSMITTER_II: i32 = 1190;

const ATTR_POWER_TRANSFER_AMOUNT: i32 = 90;
const ATTR_ENERGY_NEUTRALIZER_AMOUNT: i32 = 97;
const ATTR_DURATION: i32 = 73;

fn skills() -> HashMap<i32, u8> {
    let rdr = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
    serde_json::from_reader(rdr).unwrap()
}

fn module(type_id: i32, index: i32, state: ItemState) -> ItemModule {
    ItemModule {
        item_id: ItemID::Item(type_id),
        slot: ItemSlot {
            slot_type: ItemSlotType::High,
            index,
        },
        state,
        charge: None,
        damage_turns: 0,
    }
}

fn calculate_with_state(state: ItemState) -> Ship {
    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        security_status: 0.0,
        ship_type_id: SHIP_ARBITRATOR,
        modules: vec![
            module(MODULE_MEDIUM_ENERGY_NEUTRALIZER_II, 0, state),
            module(MODULE_MEDIUM_ENERGY_NEUTRALIZER_II, 1, state),
            module(MODULE_MEDIUM_ENERGY_NOSFERATU_II, 2, state),
            module(MODULE_MEDIUM_ENERGY_NOSFERATU_II, 3, state),
            module(MODULE_SMALL_REMOTE_CAPACITOR_TRANSMITTER_II, 4, state),
        ],
        drones: vec![],
        implants: vec![],
        boosters: vec![],
        system_buffs: vec![],
    };
    let container = FitContainer::new(fit, skills(), Default::default());
    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();
    calculate(&container, &info)
}

fn attribute(item: &Item, attribute_id: i32) -> f64 {
    item.attributes
        .get(&attribute_id)
        .and_then(|attribute| attribute.value)
        .unwrap_or_default()
}

fn module_rate(
    ship: &Ship,
    index: usize,
    amount_attribute: i32,
    rate_attribute: i32,
) -> f64 {
    let module = &ship.modules[index];
    let expected =
        attribute(module, amount_attribute) * 1000.0 / attribute(module, ATTR_DURATION);
    let actual = attribute(module, rate_attribute);
    assert_close(actual, expected);
    actual
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-6,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn active_modules_aggregate_separate_rates_without_remote_capacitor_transmitters() {
    let ship = calculate_with_state(ItemState::Active);
    let neutralizer_rate = module_rate(
        &ship,
        0,
        ATTR_ENERGY_NEUTRALIZER_AMOUNT,
        ATTR_ENERGY_NEUTRALIZER_RATE,
    ) + module_rate(
        &ship,
        1,
        ATTR_ENERGY_NEUTRALIZER_AMOUNT,
        ATTR_ENERGY_NEUTRALIZER_RATE,
    );
    let nosferatu_rate = module_rate(
        &ship,
        2,
        ATTR_POWER_TRANSFER_AMOUNT,
        ATTR_ENERGY_NOSFERATU_RATE,
    ) + module_rate(
        &ship,
        3,
        ATTR_POWER_TRANSFER_AMOUNT,
        ATTR_ENERGY_NOSFERATU_RATE,
    );

    assert!(neutralizer_rate > 0.0);
    assert!(nosferatu_rate > 0.0);
    assert_close(
        attribute(&ship.hull, ATTR_ENERGY_NEUTRALIZER_RATE),
        neutralizer_rate,
    );
    assert_close(
        attribute(&ship.hull, ATTR_ENERGY_NOSFERATU_RATE),
        nosferatu_rate,
    );
    assert_eq!(
        attribute(&ship.modules[4], ATTR_ENERGY_NEUTRALIZER_RATE),
        0.0
    );
    assert_eq!(attribute(&ship.modules[4], ATTR_ENERGY_NOSFERATU_RATE), 0.0);
}

#[test]
fn online_modules_do_not_contribute_to_rates() {
    let ship = calculate_with_state(ItemState::Online);
    assert_eq!(attribute(&ship.hull, ATTR_ENERGY_NEUTRALIZER_RATE), 0.0);
    assert_eq!(attribute(&ship.hull, ATTR_ENERGY_NOSFERATU_RATE), 0.0);
}

#[test]
fn overloaded_modules_use_modified_amount_and_duration() {
    let ship = calculate_with_state(ItemState::Overload);
    let neutralizer_rate = module_rate(
        &ship,
        0,
        ATTR_ENERGY_NEUTRALIZER_AMOUNT,
        ATTR_ENERGY_NEUTRALIZER_RATE,
    ) + module_rate(
        &ship,
        1,
        ATTR_ENERGY_NEUTRALIZER_AMOUNT,
        ATTR_ENERGY_NEUTRALIZER_RATE,
    );
    let nosferatu_rate = module_rate(
        &ship,
        2,
        ATTR_POWER_TRANSFER_AMOUNT,
        ATTR_ENERGY_NOSFERATU_RATE,
    ) + module_rate(
        &ship,
        3,
        ATTR_POWER_TRANSFER_AMOUNT,
        ATTR_ENERGY_NOSFERATU_RATE,
    );

    assert_close(
        attribute(&ship.hull, ATTR_ENERGY_NEUTRALIZER_RATE),
        neutralizer_rate,
    );
    assert_close(
        attribute(&ship.hull, ATTR_ENERGY_NOSFERATU_RATE),
        nosferatu_rate,
    );
}
