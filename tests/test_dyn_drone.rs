use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::item::{ItemID, SlotType};
use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::fit::{DynamicItem, FitContainer, ItemDrone, ItemFit, ItemState};
use eve_fit_os::protobuf::Database;

const DRONE_HOBGOBLIN: i32 = 2203;
const ATTR_DAMAGE_MULTIPLIER: i32 = 64;
const DAMAGE_FACTOR: f64 = 1.05;

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
    }
}

#[test]
fn test_dyn_drone() {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let dyn_container = {
        let mut map = HashMap::new();
        map.insert(
            1,
            DynamicItem {
                base_type: DRONE_HOBGOBLIN,
                dynamic_attributes: [(ATTR_DAMAGE_MULTIPLIER, DAMAGE_FACTOR)]
                    .into_iter()
                    .collect(),
            },
        );
        map
    };

    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let plain = calculate(
        &FitContainer::new(
            fit_with_drone(ItemID::Item(DRONE_HOBGOBLIN)),
            skill_all_5.clone(),
            Default::default(),
        ),
        &info,
    );
    let dynamic = calculate(
        &FitContainer::new(
            fit_with_drone(ItemID::Dynamic(1)),
            skill_all_5,
            dyn_container,
        ),
        &info,
    );

    let drone_attr = |ship: &eve_fit_os::calculate::Ship, item_id: ItemID| {
        ship.modules
            .iter()
            .find(|t| {
                matches!(t.slot.slot_type, SlotType::DroneBay { group_id: 0 })
                    && t.item_id == item_id
            })
            .and_then(|t| t.attributes.get(&ATTR_DAMAGE_MULTIPLIER))
            .and_then(|t| t.value)
            .unwrap()
    };

    let base = drone_attr(&plain, ItemID::Item(DRONE_HOBGOBLIN));
    let rolled = drone_attr(&dynamic, ItemID::Dynamic(1));

    println!("Drone damage multiplier: base={base}, rolled={rolled}");
    assert!(
        (rolled - base * DAMAGE_FACTOR).abs() <= base.abs() * 1e-9,
        "expected rolled == base * {DAMAGE_FACTOR}, got base={base}, rolled={rolled}",
    );
}
