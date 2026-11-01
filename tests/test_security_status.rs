//! Tests for the pilot security status input (`ItemFit::security_status`),
//! written onto the hull as `pilotSecurityStatus` (attribute 2610) in
//! calculate pass 1.
//!
//! CONCORD hulls (Pacifier / Enforcer / Marshal) carry the client-side
//! effect chain: `concordIntermediateSecStatus` (6869, 2621 <- 2610 PostMul
//! -1), `concordAssignTankBonus` (6870, 2622 <- 2621 PostMul -10), and
//! `concordSecStatusTankBonus` (6871, PostPercent 2622 onto armor repairer
//! `armorDamageAmount` (84) and shield booster `shieldBonus` (68)). Net
//! effect: local armor/shield repair amounts scale by +10% per point of
//! security status, i.e. doubled at +10 and zeroed at -10.

use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, Ship, calculate};
use eve_fit_os::fit::{
    FitContainer, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

const MARSHAL: i32 = 44996; // CONCORD battleship
const ARBITRATOR: i32 = 628; // regular hull without the sec-status chain

const MEDIUM_ARMOR_REPAIRER_II: i32 = 3530; // armorDamageAmount 368
const MEDIUM_SHIELD_BOOSTER_II: i32 = 10858; // shieldBonus 276

const PILOT_SECURITY_STATUS: i32 = 2610;
const ARMOR_DAMAGE_AMOUNT: i32 = 84;
const SHIELD_BONUS: i32 = 68;

fn skills_all_5() -> HashMap<i32, u8> {
    let rdr = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
    serde_json::from_reader(rdr).unwrap()
}

fn info() -> Database {
    Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
        .unwrap()
}

fn module(type_id: i32, slot_type: ItemSlotType, index: i32) -> ItemModule {
    ItemModule {
        item_id: ItemID::Item(type_id),
        slot: ItemSlot { slot_type, index },
        state: ItemState::Active,
        charge: None,
        damage_turns: 0,
    }
}

fn run(ship_type_id: i32, security_status: f64) -> Ship {
    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        security_status,
        ship_type_id,
        modules: vec![
            module(MEDIUM_ARMOR_REPAIRER_II, ItemSlotType::Medium, 0),
            module(MEDIUM_SHIELD_BOOSTER_II, ItemSlotType::Medium, 1),
        ],
        drones: vec![],
        implants: vec![],
        boosters: vec![],
        system_buffs: vec![],
    };
    let container = FitContainer::new(fit, skills_all_5(), Default::default());
    calculate(&container, &info())
}

fn hull_attr(ship: &Ship, attribute_id: i32) -> f64 {
    ship.hull.attributes[&attribute_id].value.unwrap()
}

fn module_attr(ship: &Ship, index: usize, attribute_id: i32) -> f64 {
    ship.modules[index].attributes[&attribute_id].value.unwrap()
}

fn assert_ratio(actual: f64, baseline: f64, ratio: f64, what: &str) {
    let expected = baseline * ratio;
    assert!(
        (actual - expected).abs() < 1e-6 * baseline.abs().max(1.0),
        "{what}: baseline {baseline}, expected {expected}, got {actual}",
    );
}

/// The CONCORD tank bonus scales local repair modules by +10% per point of
/// security status: doubled at +10, half again at +5.
#[test]
fn test_security_status_concord_tank_bonus() {
    let baseline = run(MARSHAL, 0.0);
    let maxed = run(MARSHAL, 10.0);
    let half = run(MARSHAL, 5.0);

    assert_ratio(
        hull_attr(&maxed, PILOT_SECURITY_STATUS),
        10.0,
        1.0,
        "hull pilotSecurityStatus",
    );
    for (attribute_id, index, what) in [
        (ARMOR_DAMAGE_AMOUNT, 0, "armor repair amount"),
        (SHIELD_BONUS, 1, "shield boost amount"),
    ] {
        assert_ratio(
            module_attr(&maxed, index, attribute_id),
            module_attr(&baseline, index, attribute_id),
            2.0,
            what,
        );
        assert_ratio(
            module_attr(&half, index, attribute_id),
            module_attr(&baseline, index, attribute_id),
            1.5,
            what,
        );
    }
}

/// Negative security status inverts the bonus into a penalty: at -10 the
/// PostPercent modifier is -100%, zeroing local repair amounts.
#[test]
fn test_security_status_negative_penalty() {
    let baseline = run(MARSHAL, 0.0);
    let outlaw = run(MARSHAL, -10.0);

    assert_eq!(
        module_attr(&outlaw, 0, ARMOR_DAMAGE_AMOUNT),
        0.0,
        "armor repair amount at -10",
    );
    assert_eq!(
        module_attr(&outlaw, 1, SHIELD_BONUS),
        0.0,
        "shield boost amount at -10",
    );
    assert!(
        module_attr(&baseline, 0, ARMOR_DAMAGE_AMOUNT) > 0.0,
        "baseline armor repair amount is non-zero",
    );
}

/// Out-of-range inputs are clamped to the in-game -10..=10 bounds.
#[test]
fn test_security_status_clamped() {
    let clamped_high = run(MARSHAL, 99.0);
    let maxed = run(MARSHAL, 10.0);
    assert_eq!(
        hull_attr(&clamped_high, PILOT_SECURITY_STATUS),
        hull_attr(&maxed, PILOT_SECURITY_STATUS),
        "99.0 clamps to 10.0",
    );

    let clamped_low = run(MARSHAL, -99.0);
    let outlaw = run(MARSHAL, -10.0);
    assert_eq!(
        hull_attr(&clamped_low, PILOT_SECURITY_STATUS),
        hull_attr(&outlaw, PILOT_SECURITY_STATUS),
        "-99.0 clamps to -10.0",
    );
}

/// Hulls without the sec-status effect chain (attribute 2610 exists only on
/// CONCORD/AT hulls) must be unaffected: the value is written onto the hull
/// but nothing consumes it.
#[test]
fn test_security_status_noop_on_regular_ship() {
    let baseline = run(ARBITRATOR, 0.0);
    let maxed = run(ARBITRATOR, 10.0);

    assert_ratio(
        module_attr(&maxed, 0, ARMOR_DAMAGE_AMOUNT),
        module_attr(&baseline, 0, ARMOR_DAMAGE_AMOUNT),
        1.0,
        "armor repair amount on Arbitrator",
    );
    assert_ratio(
        module_attr(&maxed, 1, SHIELD_BONUS),
        module_attr(&baseline, 1, SHIELD_BONUS),
        1.0,
        "shield boost amount on Arbitrator",
    );
}
