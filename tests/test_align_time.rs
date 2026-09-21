use std::collections::HashMap;
use std::fs::File;

use eve_fit_os::calculate::{DamageProfile, calculate};
use eve_fit_os::constant::patches::attr::ATTR_ALIGN_TIME;
use eve_fit_os::fit::{FitContainer, ItemFit};
use eve_fit_os::protobuf::Database;

/// The alignTime patch computes `-ln(0.25) * agility * mass / 1e6` via dogma
/// modifiers. Regression guard: the division by one million must be applied
/// exactly once (it was historically authored as two divisions by 1000).
#[test]
fn test_align_time() {
    let skill_all_5: HashMap<i32, u8> = {
        let rdr =
            File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/skills.json")).unwrap();
        serde_json::from_reader(rdr).unwrap()
    };

    // Buzzard; no modules, so only skill-modified agility (2.88225) and the
    // type mass (1_270_000) feed the formula.
    let fit = ItemFit {
        fighters: vec![],
        damage_profile: DamageProfile::default(),
        ship_type_id: 11192,
        modules: vec![],
        drones: vec![],
        implants: vec![],
        boosters: vec![],
        system_buffs: vec![],
    };

    let container = FitContainer::new(fit, skill_all_5, Default::default());

    let info =
        Database::init_from_root(concat!(env!("CARGO_MANIFEST_DIR"), "/data/out/pb2"))
            .unwrap();

    let out = calculate(&container, &info);

    let align_time = out
        .hull
        .attributes
        .get(&ATTR_ALIGN_TIME)
        .and_then(|t| t.value)
        .unwrap_or_default();

    // -ln(0.25) * 2.88225 * 1_270_000 / 1e6
    assert!((align_time - 5.0744715913690115).abs() < 1e-9);
}
