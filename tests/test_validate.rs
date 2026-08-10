use std::collections::HashMap;
use std::fs::File;
use std::iter::repeat_n;

use eve_fit_os::calculate::item::{FighterAbility, ItemID};
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::fit::{
    FitContainer, ItemDrone, ItemFighter, ItemFit, ItemModule, ItemSlot, ItemSlotType,
    ItemState,
};
use eve_fit_os::protobuf::Database;
use eve_fit_os::validate::{
    FighterSquadron, ValidationErrorKey, ValidationIssue, ValidationIssueKind,
    ValidationSlotType, ValidationState, validate_fit,
};

const SHIP_RIFTER: i32 = 587; // 41 MW powergrid, 130 tf CPU, 400 calibration
const SHIP_ARBITRATOR: i32 = 628; // 695 MW, 370 tf, 150 m3 drone bay, 50 Mbit/s bandwidth
const SHIP_CARRIER: i32 = 23919; // 5 fighter tubes, 3 light / 0 support / 4 heavy slots
const MODULE_HEAVY_LAUNCHER: i32 = 1877; // 77 MW, 39 tf
const MODULE_PASSIVE: i32 = 2048; // online-only module (no capacitor need)
const RIG_SMALL: i32 = 31788; // 50 calibration
const DRONE_HOBGOBLIN: i32 = 2456; // 5 Mbit/s, 5 m3
const FIGHTER_LIGHT: i32 = 40556; // group 1652 (light fighter)

fn skills() -> HashMap<i32, u8> {
    let rdr = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
    serde_json::from_reader(rdr).unwrap()
}

fn info() -> Database {
    Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
        .unwrap()
}

fn base_fit(ship_type_id: i32) -> ItemFit {
    ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id,
        modules: vec![],
        drones: vec![],
        implants: vec![],
        boosters: vec![],
    }
}

fn module(
    type_id: i32,
    slot_type: ItemSlotType,
    index: i32,
    state: ItemState,
) -> ItemModule {
    ItemModule {
        item_id: ItemID::Item(type_id),
        slot: ItemSlot { slot_type, index },
        state,
        charge: None,
    }
}

fn validate(fit: ItemFit) -> Vec<ValidationIssue> {
    let container = FitContainer::new(fit, skills(), Default::default());
    let info = info();
    let ship = calculate(&container, &info);
    validate_fit(&container, &ship, &info)
}

fn errors(issues: &[ValidationIssue]) -> Vec<&ValidationErrorKey> {
    issues
        .iter()
        .filter_map(|issue| match &issue.kind {
            ValidationIssueKind::Error(key) => Some(key),
            _ => None,
        })
        .collect()
}

#[test]
fn test_validate_powergrid_exceeded() {
    let mut fit = base_fit(SHIP_RIFTER);
    fit.modules.push(module(
        MODULE_HEAVY_LAUNCHER,
        ItemSlotType::High,
        0,
        ItemState::Active,
    ));

    let issues = validate(fit);
    let errors = errors(&issues);

    assert!(
        errors.iter().any(|key| matches!(
            key,
            ValidationErrorKey::PowergridExceeded { expected, actual }
                if (*expected - 51.25).abs() < 1e-6 && (*actual - 69.3).abs() < 1e-6
        )),
        "expected PowergridExceeded, got {issues:?}"
    );
    assert!(
        !errors
            .iter()
            .any(|key| matches!(key, ValidationErrorKey::CpuExceeded { .. })),
        "unexpected CpuExceeded, got {issues:?}"
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.slot_type == ValidationSlotType::Ship
                && issue.index.is_none()),
        "resource issues must be ship-level, got {issues:?}"
    );
}

#[test]
fn test_validate_cpu_exceeded() {
    let mut fit = base_fit(SHIP_RIFTER);
    for index in 0..6 {
        fit.modules.push(module(
            MODULE_HEAVY_LAUNCHER,
            ItemSlotType::High,
            index,
            ItemState::Active,
        ));
    }

    let issues = validate(fit);
    let errors = errors(&issues);

    assert!(
        errors.iter().any(|key| matches!(
            key,
            ValidationErrorKey::CpuExceeded { expected, actual }
                if (*expected - 162.5).abs() < 1e-6 && (*actual - 175.5).abs() < 1e-6
        )),
        "expected CpuExceeded, got {issues:?}"
    );
    assert!(
        errors
            .iter()
            .any(|key| matches!(key, ValidationErrorKey::PowergridExceeded { .. })),
        "expected PowergridExceeded, got {issues:?}"
    );
}

#[test]
fn test_validate_calibration_exceeded() {
    let mut fit = base_fit(SHIP_RIFTER);
    for index in 0..9 {
        fit.modules.push(module(
            RIG_SMALL,
            ItemSlotType::Rig,
            index,
            ItemState::Online,
        ));
    }

    let issues = validate(fit);
    let errors = errors(&issues);

    assert!(
        errors.iter().any(|key| matches!(
            key,
            ValidationErrorKey::CalibrationExceeded { expected, actual }
                if (*expected - 400.0).abs() < 1e-6 && (*actual - 450.0).abs() < 1e-6
        )),
        "expected CalibrationExceeded, got {issues:?}"
    );
}

#[test]
fn test_validate_drone_bandwidth_and_active_count() {
    let mut fit = base_fit(SHIP_ARBITRATOR);
    fit.drones = repeat_n(
        ItemDrone {
            type_id: DRONE_HOBGOBLIN,
            group_id: 10,
            state: ItemState::Active,
        },
        11,
    )
    .collect();

    let issues = validate(fit);
    let errors = errors(&issues);

    assert!(
        errors.iter().any(|key| matches!(
            key,
            ValidationErrorKey::DroneBandwidthExceeded { expected, actual }
                if (*expected - 50.0).abs() < 1e-6 && (*actual - 55.0).abs() < 1e-6
        )),
        "expected DroneBandwidthExceeded, got {issues:?}"
    );
    assert!(
        errors.iter().any(|key| matches!(
            key,
            ValidationErrorKey::TooManyActiveDrones { actual, .. } if *actual == 11
        )),
        "expected TooManyActiveDrones, got {issues:?}"
    );
    assert!(
        issues
            .iter()
            .filter(|issue| {
                matches!(
                    issue.kind,
                    ValidationIssueKind::Error(
                        ValidationErrorKey::DroneBandwidthExceeded { .. }
                            | ValidationErrorKey::TooManyActiveDrones { .. }
                    )
                )
            })
            .all(|issue| issue.slot_type == ValidationSlotType::Drone
                && issue.index.is_none()),
        "drone capacity issues must be drone-section-level, got {issues:?}"
    );
}

#[test]
fn test_validate_drone_bay_exceeded() {
    let mut fit = base_fit(SHIP_ARBITRATOR);
    fit.drones = repeat_n(
        ItemDrone {
            type_id: DRONE_HOBGOBLIN,
            group_id: 10,
            state: ItemState::Passive,
        },
        31,
    )
    .collect();

    let issues = validate(fit);
    let errors = errors(&issues);

    assert!(
        errors.iter().any(|key| matches!(
            key,
            ValidationErrorKey::DroneBayExceeded { expected, actual }
                if (*expected - 150.0).abs() < 1e-6 && (*actual - 155.0).abs() < 1e-6
        )),
        "expected DroneBayExceeded, got {issues:?}"
    );
    assert!(
        !errors.iter().any(|key| matches!(
            key,
            ValidationErrorKey::DroneBandwidthExceeded { .. }
        )),
        "passive drones must not consume bandwidth, got {issues:?}"
    );
}

#[test]
fn test_validate_fighter_tubes_exceeded() {
    let mut fit = base_fit(SHIP_CARRIER);
    fit.fighters = repeat_n(
        ItemFighter {
            type_id: FIGHTER_LIGHT,
            group_id: 0,
            ability: FighterAbility::ATTACK_MISSILE | FighterAbility::MISSILES,
        },
        6,
    )
    .collect();

    let issues = validate(fit);
    let errors = errors(&issues);

    assert!(
        errors.iter().any(|key| matches!(
            key,
            ValidationErrorKey::TooMuchFighterTube { expected, actual }
                if *expected == 5 && *actual == 6
        )),
        "expected TooMuchFighterTube, got {issues:?}"
    );
    assert!(
        errors.iter().any(|key| matches!(
            key,
            ValidationErrorKey::TooMuchFighterSquadron { category, expected, actual }
                if *category == FighterSquadron::Light && *expected == 3 && *actual == 6
        )),
        "expected TooMuchFighterSquadron(Light), got {issues:?}"
    );
}

#[test]
fn test_validate_state_exceeds_max() {
    let mut fit = base_fit(SHIP_ARBITRATOR);
    fit.modules.push(module(
        MODULE_PASSIVE,
        ItemSlotType::Low,
        0,
        ItemState::Active,
    ));

    let issues = validate(fit);

    assert!(
        issues.iter().any(|issue| {
            issue.slot_type == ValidationSlotType::Low
                && issue.index == Some(0)
                && matches!(
                    &issue.kind,
                    ValidationIssueKind::Error(ValidationErrorKey::StateExceedsMax {
                        state: ValidationState::Active,
                        max_state: ValidationState::Online,
                    })
                )
        }),
        "expected StateExceedsMax(Active > Online) on low slot 0, got {issues:?}"
    );
}

#[test]
fn test_validate_legal_fit_has_no_capacity_issues() {
    let mut fit = base_fit(SHIP_ARBITRATOR);
    for index in 0..3 {
        fit.modules.push(module(
            MODULE_HEAVY_LAUNCHER,
            ItemSlotType::High,
            index,
            ItemState::Active,
        ));
    }
    fit.drones = repeat_n(
        ItemDrone {
            type_id: DRONE_HOBGOBLIN,
            group_id: 10,
            state: ItemState::Passive,
        },
        5,
    )
    .collect();

    let issues = validate(fit);
    let errors = errors(&issues);

    assert!(
        !errors.iter().any(|key| matches!(
            key,
            ValidationErrorKey::PowergridExceeded { .. }
                | ValidationErrorKey::CpuExceeded { .. }
                | ValidationErrorKey::CalibrationExceeded { .. }
                | ValidationErrorKey::DroneBandwidthExceeded { .. }
                | ValidationErrorKey::DroneBayExceeded { .. }
                | ValidationErrorKey::TooManyActiveDrones { .. }
                | ValidationErrorKey::TooMuchFighterTube { .. }
                | ValidationErrorKey::TooMuchFighterSquadron { .. }
                | ValidationErrorKey::StateExceedsMax { .. }
        )),
        "legal fit must not produce capacity/state errors, got {issues:?}"
    );
}
