use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::ItemID;
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::fit::{
    FitContainer, ItemFit, ItemModule, ItemSlot, ItemSlotType, ItemState,
};
use eve_fit_os::protobuf::Database;

// emDamageResonance, explosiveDamageResonance, kineticDamageResonance, thermalDamageResonance
const HULL_RESONANCE: [i32; 4] = [113, 111, 109, 110];

fn calculate_with_cehe(state: ItemState) -> HashMap<i32, f64> {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 19720, // Revelation
        modules: vec![ItemModule {
            item_id: ItemID::Item(40714), // Capital Emergency Hull Energizer I
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
<<<<<<< Updated upstream
        Database::init(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2")).unwrap();
=======
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2")).unwrap();
>>>>>>> Stashed changes

    let out = calculate(&container, &info);

    HULL_RESONANCE
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

#[test]
fn test_emergency_hull_energizer() {
    let passive = calculate_with_cehe(ItemState::Passive);
    let active = calculate_with_cehe(ItemState::Active);

    for id in HULL_RESONANCE {
        let base = passive[&id];
        let energized = active[&id];

        // Capital Emergency Hull Energizer I multiplies all hull damage
        // resonances by 0.05 while active.
        assert!(
            (energized - base * 0.05).abs() < 1e-9,
            "attribute {}: expected {} * 0.05 = {}, got {}",
            id,
            base,
            base * 0.05,
            energized
        );
    }
}
