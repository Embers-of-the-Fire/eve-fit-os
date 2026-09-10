use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::fit::{
    FitContainer, ItemCharge, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

const ATTRIBUTE_DAMAGE_MULTIPLIER: i32 = 64;
const ATTRIBUTE_DAMAGE_PER_SECOND_WITHOUT_RELOAD: i32 = -14;
const ATTRIBUTE_DAMAGE_ALPHA: i32 = -13;

// Light Entropic Disintegrator I: speed 3500ms, damageMultiplier 0.8,
// damageMultiplierBonusPerCycle 0.07, damageMultiplierBonusMax 2.125.
const TYPE_LIGHT_ENTROPIC_DISINTEGRATOR: i32 = 47272;
// Tetryon Exotic Plasma S.
const CHARGE_TETRYON_EXOTIC_PLASMA_S: i32 = 47924;

fn calculate_with_spool(
    damage_turns: u32,
    state: ItemState,
) -> eve_fit_os::calculate::Ship {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 603,
        modules: vec![ItemModule {
            item_id: ItemID::Item(TYPE_LIGHT_ENTROPIC_DISINTEGRATOR),
            slot: ItemSlot {
                slot_type: ItemSlotType::High,
                index: 0,
            },
            state,
            charge: Some(ItemCharge {
                type_id: CHARGE_TETRYON_EXOTIC_PLASMA_S,
            }),
            damage_turns,
        }],
        drones: vec![],
        implants: vec![],
        boosters: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    calculate(&container, &info)
}

fn attribute(ship: &eve_fit_os::calculate::Ship, attribute_id: i32) -> f64 {
    ship.hull
        .attributes
        .get(&attribute_id)
        .and_then(|a| a.value)
        .unwrap_or_default()
}

#[test]
fn test_precursor_turret_spool_dps() {
    let base = calculate_with_spool(0, ItemState::Active);
    let base_dps = attribute(&base, ATTRIBUTE_DAMAGE_PER_SECOND_WITHOUT_RELOAD);
    assert!(
        base_dps > 0.0,
        "turret should deal damage with a charge loaded"
    );

    // 10 turns: 1 + 10 * 0.07 = 1.7x damage.
    let spooled = calculate_with_spool(10, ItemState::Active);
    let spooled_dps = attribute(&spooled, ATTRIBUTE_DAMAGE_PER_SECOND_WITHOUT_RELOAD);
    assert!(
        (spooled_dps - base_dps * 1.7).abs() < 1e-6,
        "expected {} * 1.7 = {}, got {}",
        base_dps,
        base_dps * 1.7,
        spooled_dps
    );

    // The alpha strike aggregates the module volley.
    let base_alpha = attribute(&base, ATTRIBUTE_DAMAGE_ALPHA);
    let spooled_alpha = attribute(&spooled, ATTRIBUTE_DAMAGE_ALPHA);
    assert!(
        (spooled_alpha - base_alpha * 1.7).abs() < 1e-6,
        "expected {} * 1.7 = {}, got {}",
        base_alpha,
        base_alpha * 1.7,
        spooled_alpha
    );
}

#[test]
fn test_precursor_turret_spool_capped_at_max_bonus() {
    let base = calculate_with_spool(0, ItemState::Active);
    let base_dps = attribute(&base, ATTRIBUTE_DAMAGE_PER_SECOND_WITHOUT_RELOAD);

    // 31 turns: 31 * 0.07 = 2.17 > 2.125, so the bonus caps at 2.125; any
    // further turn keeps the same damage.
    let full = calculate_with_spool(31, ItemState::Active);
    let over = calculate_with_spool(100, ItemState::Active);
    let expected = base_dps * 3.125;
    for (turns, ship) in [(31, &full), (100, &over)] {
        let dps = attribute(ship, ATTRIBUTE_DAMAGE_PER_SECOND_WITHOUT_RELOAD);
        assert!(
            (dps - expected).abs() < 1e-6,
            "turns {}: expected {}, got {}",
            turns,
            expected,
            dps
        );
    }
}

#[test]
fn test_precursor_turret_spool_requires_active() {
    let base = calculate_with_spool(0, ItemState::Online);
    let spooled = calculate_with_spool(10, ItemState::Online);
    assert_eq!(
        attribute(&base, ATTRIBUTE_DAMAGE_PER_SECOND_WITHOUT_RELOAD),
        attribute(&spooled, ATTRIBUTE_DAMAGE_PER_SECOND_WITHOUT_RELOAD),
        "an offline turret is not firing and must not spool"
    );

    // The damage multiplier itself is scaled on the module.
    let active = calculate_with_spool(10, ItemState::Active);
    let multiplier = active.modules[0]
        .attributes
        .get(&ATTRIBUTE_DAMAGE_MULTIPLIER)
        .and_then(|a| a.value)
        .unwrap_or_default();
    let base_multiplier = base.modules[0]
        .attributes
        .get(&ATTRIBUTE_DAMAGE_MULTIPLIER)
        .and_then(|a| a.value)
        .unwrap_or_default();
    assert!(
        (multiplier - base_multiplier * 1.7).abs() < 1e-9,
        "expected {} * 1.7 = {}, got {}",
        base_multiplier,
        base_multiplier * 1.7,
        multiplier
    );
}
