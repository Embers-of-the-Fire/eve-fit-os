//! Tests for user-supplied system-wide warfare buffs (environmental effects
//! such as wormhole system effects), injected via `ItemFit::system_buffs`.
//!
//! The wormhole buff definitions are hand-authored in
//! `data/patches/system_effects.yaml` (negative buff IDs); the buff strength
//! per wormhole class is supplied by the fit input. Values used here are the
//! class-6 strengths extracted from the effect beacon attribute tables
//! (reference: pyfa `eos/effects.py` system* handlers).

use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, Ship, calculate};
use eve_fit_os::fit::{
    FitContainer, ItemCharge, ItemDrone, ItemFit, ItemModule, ItemSlot, ItemSlotType,
    ItemState, ItemSystemBuff,
};
use eve_fit_os::protobuf::Database;

const SHIP: i32 = 628; // Arbitrator

// Wormhole buff IDs (see data/patches/system_effects.yaml).
const PULSAR_SHIELD_HP: i32 = -2001;
const PULSAR_SIG_RADIUS: i32 = -2002;
const PULSAR_ARMOR_RES: i32 = -2003;
const PULSAR_CAP_RECHARGE: i32 = -2004;
const BLACKHOLE_MISSILE_VELOCITY: i32 = -2008;
const BLACKHOLE_VELOCITY: i32 = -2009;
const CATACLYSMIC_ARMOR_REPAIR: i32 = -2012;
const MAGNETAR_DAMAGE: i32 = -2021;
const WOLFRAYET_SMALL_WEAPON: i32 = -2032;
const STORM_ELECTRICAL_EM_RES: i32 = -2101;
const STORM_ELECTRICAL_CAP_RECHARGE: i32 = -2102;

// Dogma attribute IDs.
const SHIELD_CAPACITY: i32 = 263;
const SIGNATURE_RADIUS: i32 = 552;
const ARMOR_EM_RESONANCE: i32 = 267;
const SHIELD_EM_RESONANCE: i32 = 271;
const RECHARGE_RATE: i32 = 55;
const MAX_VELOCITY: i32 = 37;
const EM_DAMAGE: i32 = 114;
const DAMAGE_MULTIPLIER: i32 = 64;
const ARMOR_DAMAGE_AMOUNT: i32 = 84;

// Modules and charges.
const RAPID_LIGHT_MISSILE_LAUNCHER_II: i32 = 1877;
const MJOLNIR_FURY_LIGHT_MISSILE: i32 = 2613;
const AUTOCANNON_II_200MM: i32 = 2889;
const EMP_M: i32 = 193;
const ROCKET_LAUNCHER_II: i32 = 10631;
const SCOURGE_ROCKET: i32 = 266;
const HOBGOBLIN_II: i32 = 2456;
const MEDIUM_ARMOR_REPAIRER_II: i32 = 3530;
const CORE_DEFENSE_FIELD_EXTENDER_I: i32 = 31790; // shield HP rig

fn skills_all_5() -> HashMap<i32, u8> {
    let rdr = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
    serde_json::from_reader(rdr).unwrap()
}

fn info() -> Database {
    Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
        .unwrap()
}

fn module(
    type_id: i32,
    slot_type: ItemSlotType,
    index: i32,
    charge: Option<i32>,
) -> ItemModule {
    ItemModule {
        item_id: ItemID::Item(type_id),
        slot: ItemSlot { slot_type, index },
        state: ItemState::Active,
        charge: charge.map(|type_id| ItemCharge { type_id }),
        damage_turns: 0,
    }
}

fn run(
    modules: Vec<ItemModule>,
    drones: Vec<ItemDrone>,
    system_buffs: Vec<ItemSystemBuff>,
) -> Ship {
    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: SHIP,
        modules,
        drones,
        implants: vec![],
        boosters: vec![],
        system_buffs,
    };
    let container = FitContainer::new(fit, skills_all_5(), Default::default());
    calculate(&container, &info())
}

fn buff(buff_id: i32, value: f64) -> ItemSystemBuff {
    ItemSystemBuff { buff_id, value }
}

fn hull_attr(ship: &Ship, attribute_id: i32) -> f64 {
    ship.hull.attributes[&attribute_id].value.unwrap()
}

fn module_attr(ship: &Ship, index: usize, attribute_id: i32) -> f64 {
    ship.modules[index].attributes[&attribute_id].value.unwrap()
}

fn charge_attr(ship: &Ship, index: usize, attribute_id: i32) -> f64 {
    ship.modules[index].charge.as_ref().unwrap().attributes[&attribute_id]
        .value
        .unwrap()
}

fn assert_ratio(buffed: f64, baseline: f64, ratio: f64, what: &str) {
    let expected = baseline * ratio;
    assert!(
        (buffed - expected).abs() < 1e-6 * baseline.abs().max(1.0),
        "{what}: baseline {baseline}, expected {expected}, got {buffed}",
    );
}

/// Pulsar class-6 hull effects: shield HP x2, signature radius x2, armor
/// resonances +50%, capacitor recharge time x0.5.
#[test]
fn test_system_buff_hull_attributes() {
    let buffs = vec![
        buff(PULSAR_SHIELD_HP, 2.0),
        buff(PULSAR_SIG_RADIUS, 2.0),
        buff(PULSAR_ARMOR_RES, 50.0),
        buff(PULSAR_CAP_RECHARGE, 0.5),
    ];
    let baseline = run(vec![], vec![], vec![]);
    let buffed = run(vec![], vec![], buffs);

    assert_ratio(
        hull_attr(&buffed, SHIELD_CAPACITY),
        hull_attr(&baseline, SHIELD_CAPACITY),
        2.0,
        "shield capacity",
    );
    assert_ratio(
        hull_attr(&buffed, SIGNATURE_RADIUS),
        hull_attr(&baseline, SIGNATURE_RADIUS),
        2.0,
        "signature radius",
    );
    assert_ratio(
        hull_attr(&buffed, ARMOR_EM_RESONANCE),
        hull_attr(&baseline, ARMOR_EM_RESONANCE),
        1.5,
        "armor EM resonance",
    );
    assert_ratio(
        hull_attr(&buffed, RECHARGE_RATE),
        hull_attr(&baseline, RECHARGE_RATE),
        0.5,
        "capacitor recharge rate",
    );
}

/// Multiple sources for the same buff ID aggregate by the buff's aggregate
/// mode: Maximum keeps the strongest multiplier, Minimum the lowest.
#[test]
fn test_system_buff_aggregates_by_mode() {
    // PULSAR_SHIELD_HP is Maximum: 2.0 wins over 1.3.
    let buffed = run(
        vec![],
        vec![],
        vec![buff(PULSAR_SHIELD_HP, 1.3), buff(PULSAR_SHIELD_HP, 2.0)],
    );
    let baseline = run(vec![], vec![], vec![]);
    assert_ratio(
        hull_attr(&buffed, SHIELD_CAPACITY),
        hull_attr(&baseline, SHIELD_CAPACITY),
        2.0,
        "maximum aggregation",
    );

    // PULSAR_CAP_RECHARGE is Minimum: 0.5 wins over 0.85.
    let buffed = run(
        vec![],
        vec![],
        vec![
            buff(PULSAR_CAP_RECHARGE, 0.85),
            buff(PULSAR_CAP_RECHARGE, 0.5),
        ],
    );
    assert_ratio(
        hull_attr(&buffed, RECHARGE_RATE),
        hull_attr(&baseline, RECHARGE_RATE),
        0.5,
        "minimum aggregation",
    );
}

/// Magnetar class-6 damage bonus (x2) applies to gunnery module damage
/// multipliers, missile charge damage, and drone damage multipliers.
#[test]
fn test_system_buff_magnetar_damage() {
    let modules = || {
        vec![
            module(AUTOCANNON_II_200MM, ItemSlotType::High, 0, Some(EMP_M)),
            module(
                RAPID_LIGHT_MISSILE_LAUNCHER_II,
                ItemSlotType::High,
                1,
                Some(MJOLNIR_FURY_LIGHT_MISSILE),
            ),
        ]
    };
    let drones = || {
        vec![ItemDrone {
            item_id: ItemID::Item(HOBGOBLIN_II),
            group_id: 10,
            state: ItemState::Active,
        }]
    };
    let buffs = vec![buff(MAGNETAR_DAMAGE, 2.0)];

    let baseline = run(modules(), drones(), vec![]);
    let buffed = run(modules(), drones(), buffs);

    // Turret module damage multiplier.
    assert_ratio(
        module_attr(&buffed, 0, DAMAGE_MULTIPLIER),
        module_attr(&baseline, 0, DAMAGE_MULTIPLIER),
        2.0,
        "turret damage multiplier",
    );
    // Missile charge damage.
    assert_ratio(
        charge_attr(&buffed, 1, EM_DAMAGE),
        charge_attr(&baseline, 1, EM_DAMAGE),
        2.0,
        "missile charge damage",
    );
    // Drone damage multiplier.
    assert_ratio(
        module_attr(&buffed, 2, DAMAGE_MULTIPLIER),
        module_attr(&baseline, 2, DAMAGE_MULTIPLIER),
        2.0,
        "drone damage multiplier",
    );
    // Projectile ammo does not require Missile Launcher Operation: its
    // charge attributes must not be buffed.
    assert_ratio(
        charge_attr(&buffed, 0, EM_DAMAGE),
        charge_attr(&baseline, 0, EM_DAMAGE),
        1.0,
        "projectile charge must not be buffed",
    );
}

/// Black Hole class-6: ship velocity x2, missile velocity x1.5.
#[test]
fn test_system_buff_black_hole_velocity() {
    let modules = || {
        vec![module(
            RAPID_LIGHT_MISSILE_LAUNCHER_II,
            ItemSlotType::High,
            0,
            Some(MJOLNIR_FURY_LIGHT_MISSILE),
        )]
    };
    let buffs = vec![
        buff(BLACKHOLE_VELOCITY, 2.0),
        buff(BLACKHOLE_MISSILE_VELOCITY, 1.5),
    ];
    let baseline = run(modules(), vec![], vec![]);
    let buffed = run(modules(), vec![], buffs);

    assert_ratio(
        hull_attr(&buffed, MAX_VELOCITY),
        hull_attr(&baseline, MAX_VELOCITY),
        2.0,
        "ship velocity",
    );
    assert_ratio(
        charge_attr(&buffed, 0, MAX_VELOCITY),
        charge_attr(&baseline, 0, MAX_VELOCITY),
        1.5,
        "missile velocity",
    );
}

/// Wolf Rayet class-6 small weapon damage (x3) applies to small turrets and
/// to rocket / light missile charges.
#[test]
fn test_system_buff_wolf_rayet_small_weapons() {
    let modules = || {
        vec![
            module(AUTOCANNON_II_200MM, ItemSlotType::High, 0, Some(EMP_M)),
            module(
                ROCKET_LAUNCHER_II,
                ItemSlotType::High,
                1,
                Some(SCOURGE_ROCKET),
            ),
        ]
    };
    let buffs = vec![buff(WOLFRAYET_SMALL_WEAPON, 3.0)];
    let baseline = run(modules(), vec![], vec![]);
    let buffed = run(modules(), vec![], buffs);

    assert_ratio(
        module_attr(&buffed, 0, DAMAGE_MULTIPLIER),
        module_attr(&baseline, 0, DAMAGE_MULTIPLIER),
        3.0,
        "small turret damage multiplier",
    );
    assert_ratio(
        charge_attr(&buffed, 1, EM_DAMAGE),
        charge_attr(&baseline, 1, EM_DAMAGE),
        3.0,
        "rocket charge damage",
    );
}

/// Cataclysmic Variable class-6 local armor repair penalty (x0.5).
#[test]
fn test_system_buff_cataclysmic_local_repair() {
    let modules = || {
        vec![module(
            MEDIUM_ARMOR_REPAIRER_II,
            ItemSlotType::Medium,
            0,
            None,
        )]
    };
    let buffs = vec![buff(CATACLYSMIC_ARMOR_REPAIR, 0.5)];
    let baseline = run(modules(), vec![], vec![]);
    let buffed = run(modules(), vec![], buffs);

    assert_ratio(
        module_attr(&buffed, 0, ARMOR_DAMAGE_AMOUNT),
        module_attr(&baseline, 0, ARMOR_DAMAGE_AMOUNT),
        0.5,
        "local armor repair amount",
    );
}

/// Strong metaliminal electrical storm: EM resonances +25% on hull, armor
/// and shield; capacitor recharge time x0.75.
#[test]
fn test_system_buff_electrical_storm() {
    const HULL_EM_RESONANCE: i32 = 113;
    let buffs = vec![
        buff(STORM_ELECTRICAL_EM_RES, 25.0),
        buff(STORM_ELECTRICAL_CAP_RECHARGE, 0.75),
    ];
    let baseline = run(vec![], vec![], vec![]);
    let buffed = run(vec![], vec![], buffs);

    for (attribute_id, what) in [
        (HULL_EM_RESONANCE, "hull EM resonance"),
        (ARMOR_EM_RESONANCE, "armor EM resonance"),
    ] {
        assert_ratio(
            hull_attr(&buffed, attribute_id),
            hull_attr(&baseline, attribute_id),
            1.25,
            what,
        );
    }
    // The Arbitrator's shield EM resonance is already 1.0 (0% base resist);
    // resonances clamp to [0, 1], so the penalty must not push past it.
    assert_ratio(
        hull_attr(&buffed, SHIELD_EM_RESONANCE),
        1.0,
        1.0,
        "shield EM resonance clamped",
    );
    assert_ratio(
        hull_attr(&buffed, RECHARGE_RATE),
        hull_attr(&baseline, RECHARGE_RATE),
        0.75,
        "capacitor recharge rate",
    );
}

/// Stacking penalty behavior of the hand-authored `penalized` flag:
///
/// - Penalized buffs share the penalty buckets with other penalized
///   modifiers: the Wolf Rayet (+2.0) and Magnetar (+1.0) damage buffs both
///   target a small turret's damage multiplier, so the weaker one is
///   reduced by the stacking factor.
/// - `PULSAR_SHIELD_HP` is not penalized: combined with a shield HP rig it
///   still applies at full strength.
#[test]
fn test_system_buff_stacking_penalty() {
    // Both damage buffs are penalized PostMul modifiers on the turret's
    // damage multiplier. Wolf Rayet (+2.0) is the strongest and applies in
    // full; Magnetar (+1.0) lands in penalty slot 1: x(1 + 0.869...).
    let modules = || {
        vec![module(
            AUTOCANNON_II_200MM,
            ItemSlotType::High,
            0,
            Some(EMP_M),
        )]
    };
    let baseline = run(modules(), vec![], vec![buff(WOLFRAYET_SMALL_WEAPON, 3.0)]);
    let buffed = run(
        modules(),
        vec![],
        vec![
            buff(WOLFRAYET_SMALL_WEAPON, 3.0),
            buff(MAGNETAR_DAMAGE, 2.0),
        ],
    );
    assert_ratio(
        module_attr(&buffed, 0, DAMAGE_MULTIPLIER),
        module_attr(&baseline, 0, DAMAGE_MULTIPLIER),
        1.0 + eve_fit_os::constant::PENALTY_FACTOR,
        "penalized damage multiplier",
    );

    // Shield HP buff (not penalized) + shield HP rig (penalized effect):
    // the buff applies at full strength over the rigged baseline.
    let modules = || {
        vec![module(
            CORE_DEFENSE_FIELD_EXTENDER_I,
            ItemSlotType::Rig,
            0,
            None,
        )]
    };
    let baseline = run(modules(), vec![], vec![]);
    let buffed = run(modules(), vec![], vec![buff(PULSAR_SHIELD_HP, 2.0)]);
    assert_ratio(
        hull_attr(&buffed, SHIELD_CAPACITY),
        hull_attr(&baseline, SHIELD_CAPACITY),
        2.0,
        "non-penalized shield capacity",
    );
}

/// Unknown buff IDs (e.g. a fit saved against a newer/new-to-this-snapshot
/// dbuffcollections) must never panic and must be complete no-ops: the
/// placeholder buff has no modifiers, so nothing registers on any attribute
/// in pass 4. Two entries of the same unknown ID additionally exercise the
/// `aggregate_buff` merge path.
///
/// Comparison uses a tight tolerance instead of exact equality: each `run`
/// builds a fresh `Database` and skill map, and `HashMap` iteration order
/// alone introduces last-ulp floating-point differences between runs.
#[test]
fn test_system_buff_unknown_ids_are_noop() {
    let modules = || {
        vec![
            module(
                RAPID_LIGHT_MISSILE_LAUNCHER_II,
                ItemSlotType::High,
                0,
                Some(MJOLNIR_FURY_LIGHT_MISSILE),
            ),
            module(CORE_DEFENSE_FIELD_EXTENDER_I, ItemSlotType::Rig, 1, None),
        ]
    };
    const UNKNOWN_A: i32 = 9_999_001;
    const UNKNOWN_B: i32 = 9_999_002;

    let baseline = run(modules(), vec![], vec![]);
    let buffed = run(
        modules(),
        vec![],
        vec![
            buff(UNKNOWN_A, 2.0),
            buff(UNKNOWN_A, 3.0), // duplicate: exercises the merge path
            buff(UNKNOWN_B, -50.0),
        ],
    );

    fn assert_same(buffed: Option<f64>, baseline: Option<f64>, what: String) {
        match (buffed, baseline) {
            (Some(buffed), Some(baseline)) => {
                assert!(
                    (buffed - baseline).abs() <= 1e-9 * baseline.abs().max(1.0),
                    "{what}: baseline {baseline}, got {buffed}",
                );
            }
            (buffed, baseline) => {
                assert_eq!(buffed, baseline, "{what}: presence changed");
            }
        }
    }

    for (attribute_id, baseline_attr) in &baseline.hull.attributes {
        let buffed_value = buffed.hull.attributes[attribute_id].value;
        assert_same(
            buffed_value,
            baseline_attr.value,
            format!("hull attribute {attribute_id}"),
        );
    }
    for (index, (baseline_module, buffed_module)) in baseline
        .modules
        .iter()
        .zip(buffed.modules.iter())
        .enumerate()
    {
        for (attribute_id, baseline_attr) in &baseline_module.attributes {
            let buffed_value = buffed_module.attributes[attribute_id].value;
            assert_same(
                buffed_value,
                baseline_attr.value,
                format!("module {index} attribute {attribute_id}"),
            );
        }
        let (Some(baseline_charge), Some(buffed_charge)) = (
            baseline_module.charge.as_ref(),
            buffed_module.charge.as_ref(),
        ) else {
            continue;
        };
        for (attribute_id, baseline_attr) in &baseline_charge.attributes {
            let buffed_value = buffed_charge.attributes[attribute_id].value;
            assert_same(
                buffed_value,
                baseline_attr.value,
                format!("module {index} charge attribute {attribute_id}"),
            );
        }
    }
}
