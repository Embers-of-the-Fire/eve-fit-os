//! Tests for the pilot security status input (`ItemFit::security_status`),
//! written onto the hull as `pilotSecurityStatus` (attribute 2610) in
//! calculate pass 1, clamped to the effective in-game range -10.0..=5.0
//! (+5 is the maximum reachable pilot security status; higher values only
//! exist as historical character data).
//!
//! CONCORD hulls (Pacifier / Enforcer / Marshal) carry the client-side
//! effect chain: `concordIntermediateSecStatus` (6869, 2621 <- 2610 PostMul
//! -1), `concordAssignTankBonus` (6870, 2622 <- 2621 PostMul -10), and
//! `concordSecStatusTankBonus` (6871, PostPercent 2622 onto armor repairer
//! `armorDamageAmount` (84) and shield booster `shieldBonus` (68)). Net
//! effect: local armor/shield repair amounts scale by +10% per point of
//! security status (trait: "10x pilot security status, floor 0%, ceiling
//! 50%", matching the clamped +5 maximum).
//!
//! Alliance Tournament hulls (e.g. 85062) carry the mirrored chain
//! rewarding negative security status: `ATNegativeToPositiveSecStatus`
//! (12167, 5728 <- 2610 PostMul -1), `ATIntermediateSecStatus` (12168),
//! `ATAssignShieldArmorBonus` (12166, 5727 <- 2621 PostMul -7.5), and
//! `ATFrigDmgBonus` (12165, PostPercent 5727 onto turret/missile damage).
//! Net effect: +7.5% damage per point of negative security status, up to
//! +75% at -10 (trait: "-7.5x pilot negative security status, ceiling
//! 75%").

use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, Ship, calculate};
use eve_fit_os::fit::{
    FitContainer, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

const MARSHAL: i32 = 44996; // CONCORD battleship
const AT_FRIGATE: i32 = 85062; // AT hull with the negative-sec damage chain
const ARBITRATOR: i32 = 628; // regular hull without the sec-status chain

const MEDIUM_ARMOR_REPAIRER_II: i32 = 3530; // armorDamageAmount 368
const MEDIUM_SHIELD_BOOSTER_II: i32 = 10858; // shieldBonus 276
const AUTOCANNON_II_200MM: i32 = 2889; // damageMultiplier 3.465

const PILOT_SECURITY_STATUS: i32 = 2610;
const ARMOR_DAMAGE_AMOUNT: i32 = 84;
const SHIELD_BONUS: i32 = 68;
const DAMAGE_MULTIPLIER: i32 = 64;

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

fn run(ship_type_id: i32, modules: Vec<ItemModule>, security_status: f64) -> Ship {
    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        security_status,
        ship_type_id,
        modules,
        drones: vec![],
        implants: vec![],
        boosters: vec![],
        system_buffs: vec![],
    };
    let container = FitContainer::new(fit, skills_all_5(), Default::default());
    calculate(&container, &info())
}

fn run_concord(security_status: f64) -> Ship {
    run(
        MARSHAL,
        vec![
            module(MEDIUM_ARMOR_REPAIRER_II, ItemSlotType::Medium, 0),
            module(MEDIUM_SHIELD_BOOSTER_II, ItemSlotType::Medium, 1),
        ],
        security_status,
    )
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
/// security status: +50% at the +5 maximum, +25% at +2.5.
#[test]
fn test_security_status_concord_tank_bonus() {
    let baseline = run_concord(0.0);
    let maxed = run_concord(5.0);
    let half = run_concord(2.5);

    assert_ratio(
        hull_attr(&maxed, PILOT_SECURITY_STATUS),
        5.0,
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
            1.5,
            what,
        );
        assert_ratio(
            module_attr(&half, index, attribute_id),
            module_attr(&baseline, index, attribute_id),
            1.25,
            what,
        );
    }
}

/// Negative security status inverts the CONCORD bonus into a penalty: at
/// -10 the PostPercent modifier is -100%, zeroing local repair amounts.
#[test]
fn test_security_status_negative_penalty() {
    let baseline = run_concord(0.0);
    let outlaw = run_concord(-10.0);

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

/// The AT frigate chain rewards negative security status: -7.5% damage per
/// point, i.e. +75% at -10. Positive security status inverts it into a
/// penalty (+5 -> -37.5%).
#[test]
fn test_security_status_at_damage_bonus() {
    let modules = || vec![module(AUTOCANNON_II_200MM, ItemSlotType::High, 0)];
    let baseline = run(AT_FRIGATE, modules(), 0.0);
    let outlaw = run(AT_FRIGATE, modules(), -10.0);
    let positive = run(AT_FRIGATE, modules(), 5.0);

    assert_ratio(
        module_attr(&outlaw, 0, DAMAGE_MULTIPLIER),
        module_attr(&baseline, 0, DAMAGE_MULTIPLIER),
        1.75,
        "AT turret damage at -10",
    );
    assert_ratio(
        module_attr(&positive, 0, DAMAGE_MULTIPLIER),
        module_attr(&baseline, 0, DAMAGE_MULTIPLIER),
        0.625,
        "AT turret damage at +5",
    );
}

/// Out-of-range inputs are clamped to the effective -10..=5 bounds.
#[test]
fn test_security_status_clamped() {
    let clamped_high = run_concord(99.0);
    let maxed = run_concord(5.0);
    assert_eq!(
        hull_attr(&clamped_high, PILOT_SECURITY_STATUS),
        hull_attr(&maxed, PILOT_SECURITY_STATUS),
        "99.0 clamps to 5.0",
    );

    let clamped_low = run_concord(-99.0);
    let outlaw = run_concord(-10.0);
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
    let modules = || {
        vec![
            module(MEDIUM_ARMOR_REPAIRER_II, ItemSlotType::Medium, 0),
            module(MEDIUM_SHIELD_BOOSTER_II, ItemSlotType::Medium, 1),
        ]
    };
    let baseline = run(ARBITRATOR, modules(), 0.0);
    let maxed = run(ARBITRATOR, modules(), 5.0);

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
