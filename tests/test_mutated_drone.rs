use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::{ItemID, SlotType};
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::fit::{DynamicItem, FitContainer, ItemDrone, ItemFit, ItemState};
use eve_fit_os::protobuf::Database;

const HOBGOBLIN_II: i32 = 2456;
const LIGHT_MUTATED_DRONE: i32 = 60478;
const SKILL_GALLENTE_DRONE_SPECIALIZATION: i32 = 12486;
const SKILL_MUTATED_DRONE_SPECIALIZATION: i32 = 60515;
const ATTR_HP: i32 = 9;
const ATTR_DAMAGE_MULTIPLIER: i32 = 64;
const ATTR_REQUIRED_SKILL_1: i32 = 182;
const ATTR_REQUIRED_SKILL_2: i32 = 183;
const ATTR_REQUIRED_SKILL_3: i32 = 184;
const DAMAGE_FACTOR: f64 = 1.1;

fn fit_with_drone(item_id: ItemID) -> ItemFit {
    ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 603,
        modules: vec![],
        drones: vec![ItemDrone {
            item_id,
            group_id: 0,
            state: ItemState::Active,
        }],
        implants: vec![],
        boosters: vec![],
        system_buffs: vec![],
    }
}

fn skill_all_5() -> HashMap<i32, u8> {
    let rdr = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
    serde_json::from_reader(rdr).unwrap()
}

fn drone_damage_multiplier(ship: &eve_fit_os::calculate::Ship) -> f64 {
    ship.modules
        .iter()
        .find(|t| matches!(t.slot.slot_type, SlotType::DroneBay { .. }))
        .and_then(|t| t.attributes.get(&ATTR_DAMAGE_MULTIPLIER))
        .and_then(|t| t.value)
        .unwrap()
}

// A mutated drone (dynamic item with a resulting type, e.g. Hobgoblin II
// mutated into a Light Mutated Drone) takes its required skills from the
// resulting type: Mutated Drone Specialization replaces the faction drone
// specialization skill. Its damage bonus must therefore apply, and the
// faction specialization bonus must not.
#[test]
fn test_mutated_drone_skill_from_resulting_type() {
    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let dynamic_fit = || {
        let mut dyn_container = HashMap::new();
        dyn_container.insert(
            1,
            DynamicItem {
                base_type: HOBGOBLIN_II,
                resulting_type: LIGHT_MUTATED_DRONE,
                dynamic_attributes: [(ATTR_DAMAGE_MULTIPLIER, DAMAGE_FACTOR)]
                    .into_iter()
                    .collect(),
            },
        );
        dyn_container
    };

    let run = |skills: HashMap<i32, u8>, dynamic: HashMap<i32, DynamicItem>| {
        calculate(
            &FitContainer::new(fit_with_drone(ItemID::Dynamic(1)), skills, dynamic),
            &info,
        )
    };

    // Mutated Drone Specialization V, no faction specialization.
    let mut skills_mutated_only = skill_all_5();
    skills_mutated_only.insert(SKILL_GALLENTE_DRONE_SPECIALIZATION, 0);
    // Neither specialization trained.
    let mut skills_none = skills_mutated_only.clone();
    skills_none.insert(SKILL_MUTATED_DRONE_SPECIALIZATION, 0);

    let with_mutated =
        drone_damage_multiplier(&run(skills_mutated_only, dynamic_fit()));
    let without = drone_damage_multiplier(&run(skills_none, dynamic_fit()));

    // +2% per level, level 5 => x1.10.
    assert!(
        (with_mutated / without - 1.1).abs() < 1e-9,
        "Mutated Drone Specialization V must multiply damage by 1.1, got {} / {}",
        with_mutated,
        without,
    );

    // Gallente Drone Specialization no longer applies: the mutated drone
    // requires Mutated Drone Specialization instead.
    let with_both = drone_damage_multiplier(&run(skill_all_5(), dynamic_fit()));
    assert!(
        (with_both - with_mutated).abs() < 1e-9,
        "Gallente Drone Specialization must not apply to the mutated drone: {} vs {}",
        with_both,
        with_mutated,
    );

    // The required skills come from the resulting type.
    let out = run(skill_all_5(), dynamic_fit());
    let drone = out
        .modules
        .iter()
        .find(|t| matches!(t.slot.slot_type, SlotType::DroneBay { .. }))
        .unwrap();
    let skill_req = |attr: i32| drone.attributes.get(&attr).map(|a| a.base_value);
    assert_eq!(skill_req(ATTR_REQUIRED_SKILL_1), Some(24241.0));
    assert_eq!(skill_req(ATTR_REQUIRED_SKILL_2), Some(60515.0));
    assert_eq!(skill_req(ATTR_REQUIRED_SKILL_3), Some(3436.0));

    // Non-rolled attributes take the resulting type's own value as their
    // base (skill bonuses apply on top).
    let hp = drone.attributes.get(&ATTR_HP).map(|a| a.base_value);
    assert_eq!(hp, Some(100.0));
}

// Without a resulting type (resulting_type == 0), a dynamic item keeps the
// legacy origin-only behavior.
#[test]
fn test_dynamic_drone_without_resulting_type() {
    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let mut dyn_container = HashMap::new();
    dyn_container.insert(
        1,
        DynamicItem {
            base_type: HOBGOBLIN_II,
            resulting_type: 0,
            dynamic_attributes: [(ATTR_DAMAGE_MULTIPLIER, DAMAGE_FACTOR)]
                .into_iter()
                .collect(),
        },
    );

    let plain = calculate(
        &FitContainer::new(
            fit_with_drone(ItemID::Item(HOBGOBLIN_II)),
            skill_all_5(),
            Default::default(),
        ),
        &info,
    );
    let dynamic = calculate(
        &FitContainer::new(
            fit_with_drone(ItemID::Dynamic(1)),
            skill_all_5(),
            dyn_container,
        ),
        &info,
    );

    let base = drone_damage_multiplier(&plain);
    let rolled = drone_damage_multiplier(&dynamic);
    assert!(
        (rolled - base * DAMAGE_FACTOR).abs() <= base.abs() * 1e-9,
        "expected rolled == base * {DAMAGE_FACTOR}, got base={base}, rolled={rolled}",
    );
}
