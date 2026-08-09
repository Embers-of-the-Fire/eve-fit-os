use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::fit::{
    FitContainer, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

// emDamageResonance, explosiveDamageResonance, kineticDamageResonance, thermalDamageResonance
const ARMOR_RESONANCE: [i32; 4] = [267, 268, 269, 270];

const TYPE_REACTIVE_ARMOR_HARDENER: i32 = 4403;
const TYPE_XARASIER_REACTIVE_ARMOR_HARDENER: i32 = 88709;

fn calculate_with_rah(
    type_id: i32,
    state: ItemState,
    damage_profile: DamageProfile,
) -> HashMap<i32, f64> {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile,
        // Merlin: base armor resonances are 0.5 / 0.9 / 0.75 / 0.55, so the
        // damage the RAH reacts to is not uniform even with a uniform profile.
        ship_type_id: 603,
        modules: vec![ItemModule {
            item_id: ItemID::Item(type_id),
            slot: ItemSlot {
                slot_type: ItemSlotType::Low,
                index: 0,
            },
            state,
            charge: None,
        }],
        drones: vec![],
        implants: vec![],
        boosters: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let out = calculate(&container, &info);

    ARMOR_RESONANCE
        .into_iter()
        .map(|id| {
            (
                id,
                out.hull
                    .attributes
                    .get(&id)
                    .and_then(|a| a.value)
                    .unwrap_or_default(),
            )
        })
        .collect()
}

fn assert_factors(
    base: &HashMap<i32, f64>,
    adapted: &HashMap<i32, f64>,
    factors: [f64; 4],
) {
    for (id, factor) in ARMOR_RESONANCE.into_iter().zip(factors) {
        assert!(
            (adapted[&id] - base[&id] * factor).abs() < 1e-9,
            "attribute {}: expected {} * {} = {}, got {}",
            id,
            base[&id],
            factor,
            base[&id] * factor,
            adapted[&id]
        );
    }
}

fn assert_pool(base: &HashMap<i32, f64>, adapted: &HashMap<i32, f64>, pool: f64) {
    let total: f64 = ARMOR_RESONANCE
        .into_iter()
        .map(|id| 1.0 - adapted[&id] / base[&id])
        .sum();
    assert!(
        (total - pool).abs() < 1e-9,
        "expected total resistance pool {}, got {}",
        pool,
        total
    );
}

#[test]
fn test_reactive_armor_hardener_uniform() {
    let profile = DamageProfile::default();

    let base =
        calculate_with_rah(TYPE_REACTIVE_ARMOR_HARDENER, ItemState::Passive, profile);
    let active =
        calculate_with_rah(TYPE_REACTIVE_ARMOR_HARDENER, ItemState::Active, profile);

    // The RAH adapts to post-resistance damage: with a uniform profile on a
    // Merlin the highest damage taken is explosive/kinetic, so the 60% pool
    // concentrates there and drains fully from EM and thermal.
    assert_factors(&base, &active, [1.0, 0.655, 0.745, 1.0]);
    assert_pool(&base, &active, 0.60);
}

#[test]
fn test_reactive_armor_hardener_single_type() {
    let profile = DamageProfile {
        em: 1.0,
        explosive: 0.0,
        kinetic: 0.0,
        thermal: 0.0,
    };

    let base =
        calculate_with_rah(TYPE_REACTIVE_ARMOR_HARDENER, ItemState::Passive, profile);
    let active =
        calculate_with_rah(TYPE_REACTIVE_ARMOR_HARDENER, ItemState::Active, profile);

    // Pure EM: the whole 60% pool concentrates on EM, the rest drops to zero.
    assert_factors(&base, &active, [0.40, 1.0, 1.0, 1.0]);
    assert_pool(&base, &active, 0.60);
}

#[test]
fn test_reactive_armor_hardener_dual_type() {
    let profile = DamageProfile {
        em: 0.5,
        explosive: 0.0,
        kinetic: 0.5,
        thermal: 0.0,
    };

    let base =
        calculate_with_rah(TYPE_REACTIVE_ARMOR_HARDENER, ItemState::Passive, profile);
    let active =
        calculate_with_rah(TYPE_REACTIVE_ARMOR_HARDENER, ItemState::Active, profile);

    // 50/50 EM/kinetic: both climb to 30%, the rest drops to zero.
    assert_factors(&base, &active, [0.70, 1.0, 0.70, 1.0]);
    assert_pool(&base, &active, 0.60);
}

#[test]
fn test_reactive_armor_hardener_xarasier() {
    let uniform = DamageProfile::default();

    let base = calculate_with_rah(
        TYPE_XARASIER_REACTIVE_ARMOR_HARDENER,
        ItemState::Passive,
        uniform,
    );
    let active = calculate_with_rah(
        TYPE_XARASIER_REACTIVE_ARMOR_HARDENER,
        ItemState::Active,
        uniform,
    );

    // The Xarasier variant has a larger 64% pool, again concentrated on the
    // highest damage taken.
    assert_factors(&base, &active, [1.0, 0.62, 0.755, 0.985]);
    assert_pool(&base, &active, 0.64);

    let single = DamageProfile {
        em: 1.0,
        explosive: 0.0,
        kinetic: 0.0,
        thermal: 0.0,
    };
    let active = calculate_with_rah(
        TYPE_XARASIER_REACTIVE_ARMOR_HARDENER,
        ItemState::Active,
        single,
    );

    // Pure EM: the whole 64% pool concentrates on EM.
    assert_factors(&base, &active, [0.36, 1.0, 1.0, 1.0]);
    assert_pool(&base, &active, 0.64);
}
