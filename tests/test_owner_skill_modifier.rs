use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::{ItemID, SlotType};
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::fit::{
    FitContainer, ItemDrone, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

const SIN_TYPE_ID: i32 = 22430;
const RAILGUN_425_TYPE_ID: i32 = 574;
const KHRYSEOS_MFS_TYPE_ID: i32 = 94020;
const HOBGOBLIN_II_TYPE_ID: i32 = 2456;
const ATTR_DAMAGE_MULTIPLIER: i32 = 64;

// The Federation Navy 'Khryseos' Magnetic Field Stabilizer requires the
// Drones skill as its secondary skill. Owner-gated (OwnerRequiredSkillModifier)
// drone bonuses - the Sin's hull drone damage bonus and the Khryseos' own
// drone damage bonus - must only apply to drones, fighters and charges, never
// to fitted modules. Otherwise the module's turret damage bonus (attribute
// 64) is multiplied by every drone damage source and turret DPS explodes.
#[test]
fn test_owner_skill_modifier_not_applied_to_modules() {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: SIN_TYPE_ID,
        modules: [ItemModule {
            item_id: ItemID::Item(RAILGUN_425_TYPE_ID),
            slot: ItemSlot {
                slot_type: ItemSlotType::High,
                index: 0,
            },
            state: ItemState::Active,
            charge: None,
            damage_turns: 0,
        }]
        .into_iter()
        .chain((0..4).map(|index| ItemModule {
            item_id: ItemID::Item(KHRYSEOS_MFS_TYPE_ID),
            slot: ItemSlot {
                slot_type: ItemSlotType::Low,
                index,
            },
            state: ItemState::Active,
            charge: None,
            damage_turns: 0,
        }))
        .collect(),
        drones: vec![ItemDrone {
            item_id: ItemID::Item(HOBGOBLIN_II_TYPE_ID),
            group_id: 10,
            state: ItemState::Active,
        }],
        implants: vec![],
        boosters: vec![],
        system_buffs: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let out = calculate(&container, &info);

    let mfs = out
        .modules
        .iter()
        .find(|t| t.item_id == ItemID::Item(KHRYSEOS_MFS_TYPE_ID))
        .expect("Khryseos MFS not found");
    let mfs_damage = mfs
        .attributes
        .get(&ATTR_DAMAGE_MULTIPLIER)
        .and_then(|t| t.value)
        .expect("Khryseos MFS has no damageMultiplier");
    assert!(
        (mfs_damage - 1.1).abs() < 1e-9,
        "Khryseos MFS damageMultiplier must stay 1.1, got {}",
        mfs_damage,
    );

    let railgun = out
        .modules
        .iter()
        .find(|t| t.item_id == ItemID::Item(RAILGUN_425_TYPE_ID))
        .expect("425mm Railgun not found");
    let railgun_damage = railgun
        .attributes
        .get(&ATTR_DAMAGE_MULTIPLIER)
        .and_then(|t| t.value)
        .expect("425mm Railgun has no damageMultiplier");
    // 3.025 base, 4x stacking-penalized x1.1 from the stabilizers, x2 from
    // the Sin's hybrid turret bonus (Gallente Battleship V), +25% from
    // Large Hybrid Turret V and +15% from Surgical Strike V.
    assert!(
        (railgun_damage - 11.302309201805764).abs() < 1e-9,
        "425mm Railgun damageMultiplier must be ~11.30, got {}",
        railgun_damage,
    );

    // Drones must still receive the drone damage bonuses: the Sin's hull
    // bonus, the 4x stacking-penalized +15% from the stabilizers and the
    // drone skill bonuses.
    let drone = out
        .modules
        .iter()
        .find(|t| matches!(t.slot.slot_type, SlotType::DroneBay { .. }))
        .expect("drone not found");
    let drone_damage = drone
        .attributes
        .get(&ATTR_DAMAGE_MULTIPLIER)
        .and_then(|t| t.value)
        .expect("drone has no damageMultiplier");
    assert!(
        (drone_damage - 8.73818816517104).abs() < 1e-9,
        "Hobgoblin II damageMultiplier must be ~8.74, got {}",
        drone_damage,
    );
}
