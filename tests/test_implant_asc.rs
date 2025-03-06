use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::constant::patches::attr::ATTR_WARP_SPEED;
use eve_fit_os::fit::{FitContainer, ItemFit, ItemImplant};
use eve_fit_os::protobuf::Database;

#[test]
fn test_implant_asc() {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 628,
        modules: vec![],
        drones: vec![],
        implants: vec![
            ItemImplant {
                type_id: 33516,
                index: 0,
            },
            ItemImplant {
                type_id: 33525,
                index: 0,
            },
        ],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2")).unwrap();

    let out = calculate(&container, &info);

    println!(
        "warp: {:?}",
        out.hull.attributes.get(&ATTR_WARP_SPEED).unwrap().value
    );
    for implant in &out.implants {
        print!("{:?} ", implant.attributes.get(&624).unwrap().value);
    }
    println!();
}
