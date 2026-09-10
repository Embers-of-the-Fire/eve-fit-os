use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::constant::patches::attr::ATTR_ARMOR_REPAIR_RATE;
use eve_fit_os::fit::{
    FitContainer, ItemCharge, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;
use eve_fit_os::provider::InfoProvider;

#[test]
fn test_warfare() {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 23919,
        modules: vec![
            ItemModule {
                item_id: ItemID::Item(43552),
                slot: ItemSlot {
                    slot_type: ItemSlotType::High,
                    index: 0,
                },
                // state: ItemState::Active,
                state: ItemState::Active,
                charge: Some(ItemCharge { type_id: 42833 }),
                damage_turns: 0,
            },
            ItemModule {
                item_id: ItemID::Item(3530),
                slot: ItemSlot {
                    slot_type: ItemSlotType::Low,
                    index: 0,
                },
                state: ItemState::Active,
                charge: None,
                damage_turns: 0,
            },
        ],
        drones: vec![],
        implants: vec![],
        boosters: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let out = calculate(&container, &info);

    let rep = out
        .hull
        .attributes
        .get(&ATTR_ARMOR_REPAIR_RATE)
        .and_then(|t| t.value)
        .unwrap_or_default();
    println!("hull repair: {:?}", rep);
}

/// Regression test for <https://github.com/Embers-of-the-Fire/eve-fit-assistant/issues/452>.
///
/// Warfare buff `itemModifiers` describe ship-hull attributes (e.g. armor
/// resonances 267-270 for the resistance charge). They must not be injected
/// into module charges; otherwise every burst charge displays the resistance
/// charge's attributes whenever any burst module loads it.
#[test]
fn test_warfare_charge_attributes_not_polluted() {
    const ARMOR_RESONANCES: [i32; 4] = [267, 268, 269, 270];
    const ARMORED_COMMAND_BURST_II: i32 = 43552;
    // Armored command burst charges: resistance / repair amount / repair rate.
    const CHARGES: [i32; 3] = [42832, 42833, 42834];

    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 23919,
        modules: CHARGES
            .iter()
            .enumerate()
            .map(|(index, type_id)| ItemModule {
                item_id: ItemID::Item(ARMORED_COMMAND_BURST_II),
                slot: ItemSlot {
                    slot_type: ItemSlotType::High,
                    index: index as i32,
                },
                state: ItemState::Active,
                charge: Some(ItemCharge { type_id: *type_id }),
                damage_turns: 0,
            })
            .collect(),
        drones: vec![],
        implants: vec![],
        boosters: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let out = calculate(&container, &info);

    // Charges must only carry their own dogma attributes; hull attributes
    // (armor resonances) must never leak into them.
    for (index, module) in out.modules.iter().enumerate() {
        let charge = module.charge.as_ref().expect("charge must be loaded");
        for attribute_id in ARMOR_RESONANCES {
            assert!(
                !charge.attributes.contains_key(&attribute_id),
                "charge of module {index} (type {}) must not contain hull attribute {attribute_id}",
                CHARGES[index],
            );
        }
    }

    // The buff itself must still reach the hull: the resistance charge's buff
    // changes the ship's armor EM resonance away from its base value.
    let base_em_resonance = info
        .get_dogma_attributes(23919)
        .iter()
        .find(|a| a.attribute_id == 267)
        .expect("ship must define armor EM resonance")
        .value;
    let hull_em_resonance = out
        .hull
        .attributes
        .get(&267)
        .and_then(|t| t.value)
        .unwrap_or_default();
    assert!(
        (hull_em_resonance - base_em_resonance).abs() > f64::EPSILON,
        "warfare buff must still modify hull resonance: base {base_em_resonance}, got {hull_em_resonance}",
    );
}

/// Warfare buff `itemModifiers` describe attributes of the buff holder — the
/// ship hull — and must not be injected into equipped items.
///
/// Using the shield resistance buff (Shield Harmonizing Charge, buff id 10,
/// `itemModifiers` = shield resonances 271-274, PostPercent):
///
/// 1. A Damage Control II carries its own resonance attributes 271-274 and
///    forwards them to the hull via effect 2302. If the buff were injected
///    into the module, the hull would receive `base * buff * (module * buff)`
///    instead of `base * module * buff` — the buff would double-dip and EHP
///    would be inflated.
/// 2. A rig (Medium Core Defense Field Extender I) does not carry resonance
///    attributes at all and must not gain spurious ones.
/// 3. The hull must still receive the buff exactly once.
#[test]
fn test_warfare_item_modifiers_apply_to_hull_only() {
    const SHIELD_RESONANCES: [i32; 4] = [271, 272, 273, 274];
    const SHIP: i32 = 23919; // Damnation
    const SHIELD_COMMAND_BURST_II: i32 = 43555;
    const SHIELD_HARMONIZING_CHARGE: i32 = 42695;
    const DAMAGE_CONTROL_II: i32 = 2048;
    const CORE_DEFENSE_FIELD_EXTENDER_I: i32 = 31790; // rig
    const WARFARE_BUFF_1_VALUE: i32 = 2469;

    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let make_fit = |with_burst: bool| {
        let mut modules = vec![];
        if with_burst {
            modules.push(ItemModule {
                item_id: ItemID::Item(SHIELD_COMMAND_BURST_II),
                slot: ItemSlot {
                    slot_type: ItemSlotType::High,
                    index: 0,
                },
                state: ItemState::Active,
                charge: Some(ItemCharge {
                    type_id: SHIELD_HARMONIZING_CHARGE,
                }),
                damage_turns: 0,
            });
        }
        modules.push(ItemModule {
            item_id: ItemID::Item(DAMAGE_CONTROL_II),
            slot: ItemSlot {
                slot_type: ItemSlotType::Low,
                index: 0,
            },
            state: ItemState::Active,
            charge: None,
            damage_turns: 0,
        });
        modules.push(ItemModule {
            item_id: ItemID::Item(CORE_DEFENSE_FIELD_EXTENDER_I),
            slot: ItemSlot {
                slot_type: ItemSlotType::Rig,
                index: 0,
            },
            state: ItemState::Passive,
            charge: None,
            damage_turns: 0,
        });

        FitContainer::new(
            ItemFit {
                fighters: vec![],
                damage_profile: DamageProfile::default(),
                ship_type_id: SHIP,
                modules,
                drones: vec![],
                implants: vec![],
                boosters: vec![],
            },
            skill_all_5.clone(),
            Default::default(),
        )
    };

    let buffed = calculate(&make_fit(true), &info);
    let baseline = calculate(&make_fit(false), &info);

    // With the burst fitted, module order is: burst, DCU, rig.
    let dcu = &buffed.modules[1];
    assert_eq!(
        dcu.item_id,
        ItemID::Item(DAMAGE_CONTROL_II),
        "unexpected module order",
    );

    // 1. The DCU's own resonance attributes must stay at their base values;
    //    the buff must not be injected into the module (no double-dip).
    for attribute_id in SHIELD_RESONANCES {
        let base = info
            .get_dogma_attributes(DAMAGE_CONTROL_II)
            .iter()
            .find(|a| a.attribute_id == attribute_id)
            .expect("DCU must define the resonance attribute")
            .value;
        let value = dcu
            .attributes
            .get(&attribute_id)
            .and_then(|t| t.value)
            .unwrap_or_default();
        assert!(
            (value - base).abs() < f64::EPSILON,
            "DCU attribute {attribute_id} must not be buffed: base {base}, got {value}",
        );
    }

    // 2. The rig must not gain spurious resonance attributes.
    let rig = &buffed.modules[2];
    assert_eq!(
        rig.item_id,
        ItemID::Item(CORE_DEFENSE_FIELD_EXTENDER_I),
        "unexpected module order",
    );
    for attribute_id in SHIELD_RESONANCES {
        assert!(
            !rig.attributes.contains_key(&attribute_id),
            "rig must not contain hull attribute {attribute_id}",
        );
    }

    // 3. The hull must receive the buff exactly once: buffed resonance
    //    equals the unbuffed resonance scaled by the buff value.
    let buff_value = buffed.modules[0]
        .attributes
        .get(&WARFARE_BUFF_1_VALUE)
        .and_then(|t| t.value)
        .expect("burst module must carry warfareBuff1Value");
    assert!(buff_value != 0.0, "warfare buff value must be non-zero");
    for attribute_id in SHIELD_RESONANCES {
        let buffed_value = buffed
            .hull
            .attributes
            .get(&attribute_id)
            .and_then(|t| t.value)
            .unwrap_or_default();
        let baseline_value = baseline
            .hull
            .attributes
            .get(&attribute_id)
            .and_then(|t| t.value)
            .unwrap_or_default();
        let expected = baseline_value * (1.0 + buff_value / 100.0);
        assert!(
            (buffed_value - expected).abs() < 1e-9,
            "hull attribute {attribute_id} must be buffed exactly once: \
             baseline {baseline_value}, expected {expected}, got {buffed_value}",
        );
    }
}
