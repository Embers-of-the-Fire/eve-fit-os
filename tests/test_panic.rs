use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::fit::{
    FitContainer, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;
use eve_fit_os::validate::{
    ValidationErrorKey, ValidationIssue, ValidationIssueKind, validate_fit,
};

const SHIP_RORQUAL: i32 = 28352;
const MODULE_PANIC: i32 = 42522; // Pulse Activated Nexus Invulnerability Core
const MODULE_CAPITAL_INDUSTRIAL_CORE_II: i32 = 42890;
const MODULE_COMPRESSOR: i32 = 62632; // Capital Asteroid Ore Compressor I
const MODULE_DAMAGE_CONTROL_II: i32 = 2048;

const SHIELD_RESONANCES: [i32; 4] = [271, 272, 273, 274];
const SHIELD_RECHARGE_RATE: i32 = 479;

fn skills() -> HashMap<i32, u8> {
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
    state: ItemState,
) -> ItemModule {
    ItemModule {
        item_id: ItemID::Item(type_id),
        slot: ItemSlot { slot_type, index },
        state,
        charge: None,
        damage_turns: 0,
    }
}

fn fit_with(modules: Vec<ItemModule>) -> ItemFit {
    ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: SHIP_RORQUAL,
        modules,
        drones: vec![],
        implants: vec![],
        boosters: vec![],
        system_buffs: vec![],
    }
}

fn validate(fit: ItemFit) -> Vec<ValidationIssue> {
    let container = FitContainer::new(fit, skills(), Default::default());
    let info = info();
    let ship = calculate(&container, &info);
    validate_fit(&container, &ship, &info)
}

/// The PANIC buff (patched in by `data/patches/panic.yaml`) must scale all
/// shield resonances to 0 (full damage immunity) and cut the shield recharge
/// time by 90% while the module is active.
#[test]
fn test_panic_buff_applies_when_active() {
    let info = info();

    let online = calculate(
        &FitContainer::new(
            fit_with(vec![module(
                MODULE_PANIC,
                ItemSlotType::High,
                0,
                ItemState::Online,
            )]),
            skills(),
            Default::default(),
        ),
        &info,
    );
    let active = calculate(
        &FitContainer::new(
            fit_with(vec![module(
                MODULE_PANIC,
                ItemSlotType::High,
                0,
                ItemState::Active,
            )]),
            skills(),
            Default::default(),
        ),
        &info,
    );

    for attribute_id in SHIELD_RESONANCES {
        let base = online
            .hull
            .attributes
            .get(&attribute_id)
            .and_then(|a| a.value)
            .expect("hull must define shield resonance");
        assert!(
            base > 0.0,
            "unexpected Rorqual base resonance {attribute_id}"
        );

        let buffed = active
            .hull
            .attributes
            .get(&attribute_id)
            .and_then(|a| a.value)
            .expect("hull must define shield resonance");
        assert_eq!(
            buffed, 0.0,
            "active PANIC must scale resonance {attribute_id} to 0 (base {base})",
        );
    }

    let recharge_online = online
        .hull
        .attributes
        .get(&SHIELD_RECHARGE_RATE)
        .and_then(|a| a.value)
        .expect("hull must define shield recharge rate");
    let recharge_active = active
        .hull
        .attributes
        .get(&SHIELD_RECHARGE_RATE)
        .and_then(|a| a.value)
        .expect("hull must define shield recharge rate");
    let expected = recharge_online * 0.1;
    assert!(
        (recharge_active - expected).abs() < 1e-6,
        "active PANIC must cut shield recharge time by 90%: \
         online {recharge_online}, expected {expected}, got {recharge_active}",
    );
}

/// The resonance immunity must hold even when combined with other
/// stacking-penalized resonance modifiers (e.g. a Damage Control).
#[test]
fn test_panic_buff_with_stacked_resonance_modifiers() {
    let info = info();
    let out = calculate(
        &FitContainer::new(
            fit_with(vec![
                module(MODULE_PANIC, ItemSlotType::High, 0, ItemState::Active),
                module(
                    MODULE_DAMAGE_CONTROL_II,
                    ItemSlotType::Low,
                    0,
                    ItemState::Active,
                ),
            ]),
            skills(),
            Default::default(),
        ),
        &info,
    );

    for attribute_id in SHIELD_RESONANCES {
        let buffed = out
            .hull
            .attributes
            .get(&attribute_id)
            .and_then(|a| a.value)
            .expect("hull must define shield resonance");
        assert_eq!(
            buffed, 0.0,
            "active PANIC must scale resonance {attribute_id} to 0 even when stacked",
        );
    }
}

/// Activating the PANIC module or a compressor without an active industrial
/// core must be flagged; an active industrial core must clear the issue.
#[test]
fn test_validate_requires_active_industrial_core() {
    let requires_core = |issues: &[ValidationIssue]| {
        issues.iter().any(|issue| {
            matches!(
                issue.kind,
                ValidationIssueKind::Error(
                    ValidationErrorKey::RequiresActiveIndustrialCore
                )
            )
        })
    };

    // PANIC active without an industrial core: flagged.
    let issues = validate(fit_with(vec![module(
        MODULE_PANIC,
        ItemSlotType::High,
        0,
        ItemState::Active,
    )]));
    assert!(
        requires_core(&issues),
        "active PANIC without industrial core must be flagged, got {issues:?}",
    );

    // PANIC online (not activated): not flagged.
    let issues = validate(fit_with(vec![module(
        MODULE_PANIC,
        ItemSlotType::High,
        0,
        ItemState::Online,
    )]));
    assert!(
        !requires_core(&issues),
        "online PANIC must not be flagged, got {issues:?}",
    );

    // PANIC active with an active industrial core: not flagged.
    let issues = validate(fit_with(vec![
        module(MODULE_PANIC, ItemSlotType::High, 0, ItemState::Active),
        module(
            MODULE_CAPITAL_INDUSTRIAL_CORE_II,
            ItemSlotType::High,
            1,
            ItemState::Active,
        ),
    ]));
    assert!(
        !requires_core(&issues),
        "active PANIC with active industrial core must not be flagged, got {issues:?}",
    );

    // Compressor active without an industrial core: flagged.
    let issues = validate(fit_with(vec![module(
        MODULE_COMPRESSOR,
        ItemSlotType::High,
        0,
        ItemState::Active,
    )]));
    assert!(
        requires_core(&issues),
        "active compressor without industrial core must be flagged, got {issues:?}",
    );

    // Compressor active with an online (not activated) industrial core:
    // still flagged.
    let issues = validate(fit_with(vec![
        module(MODULE_COMPRESSOR, ItemSlotType::High, 0, ItemState::Active),
        module(
            MODULE_CAPITAL_INDUSTRIAL_CORE_II,
            ItemSlotType::High,
            1,
            ItemState::Online,
        ),
    ]));
    assert!(
        requires_core(&issues),
        "active compressor with only an online industrial core must be flagged, got {issues:?}",
    );

    // Compressor active with an active industrial core: not flagged.
    let issues = validate(fit_with(vec![
        module(MODULE_COMPRESSOR, ItemSlotType::High, 0, ItemState::Active),
        module(
            MODULE_CAPITAL_INDUSTRIAL_CORE_II,
            ItemSlotType::High,
            1,
            ItemState::Active,
        ),
    ]));
    assert!(
        !requires_core(&issues),
        "active compressor with active industrial core must not be flagged, got {issues:?}",
    );
}
