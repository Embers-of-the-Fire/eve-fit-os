use crate::calculate::Ship;
use crate::provider::{FitProvider, InfoProvider};

const EFFECT_ADAPTIVE_ARMOR_HARDENER: i32 = 4928;
const ATTR_RESISTANCE_SHIFT_AMOUNT: i32 = 1849;
// armorEmDamageResonance, armorExplosiveDamageResonance,
// armorKineticDamageResonance, armorThermalDamageResonance
const ARMOR_RESONANCE: [i32; 4] = [267, 268, 269, 270];

const MAX_CYCLES: usize = 50;
const LOOP_TOLERANCE: f64 = 1e-6;

/// Simulate the reactive armor hardener adaptation cycles against the damage
/// profile until the resistance profile stops changing or enters a loop.
///
/// Ported from pyfa's Effect4928 (adaptiveArmorHardener). The initial
/// resistance pool is read from the module itself, so variants with a
/// different total (e.g. Xarasier, 64%) are supported.
fn simulate_adaptation(
    base_damage_taken: [f64; 4],
    initial: [f64; 4],
    shift: f64,
) -> [f64; 4] {
    let mut resistance = initial;
    let mut history: Vec<[f64; 4]> = Vec::new();
    let mut loop_start = None;

    for _ in 0..MAX_CYCLES {
        // Stable sort on ascending damage taken; index order emulates the
        // ingame tie-breaking when different types took the same damage.
        let mut order = [0usize, 1, 2, 3];
        order.sort_by(|&a, &b| {
            (base_damage_taken[a] * resistance[a])
                .total_cmp(&(base_damage_taken[b] * resistance[b]))
        });
        let damage = |i: usize| base_damage_taken[order[i]] * resistance[order[i]];

        let mut change = [0.0; 4];
        if damage(2) == 0.0 {
            // One damage type: it takes resistance from the other three.
            change[order[0]] = 1.0 - resistance[order[0]];
            change[order[1]] = 1.0 - resistance[order[1]];
            change[order[2]] = 1.0 - resistance[order[2]];
            change[order[3]] =
                -(change[order[0]] + change[order[1]] + change[order[2]]);
        } else if damage(1) == 0.0 {
            // Two damage types: they take resistance equally from the other two.
            change[order[0]] = 1.0 - resistance[order[0]];
            change[order[1]] = 1.0 - resistance[order[1]];
            let rest = -(change[order[0]] + change[order[1]]) / 2.0;
            change[order[2]] = rest;
            change[order[3]] = rest;
        } else {
            // Three or four damage types: the two types taking the most
            // damage drain resistance from the two taking the least.
            change[order[0]] = shift.min(1.0 - resistance[order[0]]);
            change[order[1]] = shift.min(1.0 - resistance[order[1]]);
            let rest = -(change[order[0]] + change[order[1]]) / 2.0;
            change[order[2]] = rest;
            change[order[3]] = rest;
        }

        for i in 0..4 {
            resistance[i] += change[i];
        }

        for (index, previous) in history.iter().enumerate() {
            if previous
                .iter()
                .zip(resistance.iter())
                .all(|(a, b)| (a - b).abs() <= LOOP_TOLERANCE)
            {
                loop_start = Some(index);
                break;
            }
        }
        if loop_start.is_some() {
            break;
        }

        history.push(resistance);
    }

    // Average the profiles in the loop, or the last 20 if no loop was found.
    let cycles = match loop_start {
        Some(start) => &history[start..],
        None => &history[history.len().saturating_sub(20)..],
    };
    let mut average = [0.0; 4];
    for cycle in cycles {
        for i in 0..4 {
            average[i] += cycle[i];
        }
    }
    for value in &mut average {
        *value = (*value / cycles.len() as f64 * 1000.0).round() / 1000.0;
    }
    average
}

/// The reactive armor hardener shifts its resistance towards the incoming
/// damage. Its dogma attributes only contain the initial (uniform) values, so
/// we compute the adapted equilibrium from the damage profile here and patch
/// the module attributes before any effects are applied.
pub fn attribute_reactive_armor_hardener(
    fit: &impl FitProvider,
    info: &impl InfoProvider,
    ship: &mut Ship,
) {
    let profile = ship.damage_profile;
    let damage = [
        profile.em,
        profile.explosive,
        profile.kinetic,
        profile.thermal,
    ];
    if damage.iter().sum::<f64>() <= 0.0 {
        return;
    }

    for module in ship.modules.iter_mut() {
        // The adaptiveArmorHardener effect is active-only.
        if !module.state.is_active() {
            continue;
        }

        let type_id = module.item_id.as_type_id(fit);
        if !info
            .get_dogma_effects(type_id)
            .iter()
            .any(|effect| effect.effect_id == EFFECT_ADAPTIVE_ARMOR_HARDENER)
        {
            continue;
        }

        let Some(initial) = ARMOR_RESONANCE
            .iter()
            .map(|id| module.attributes.get(id).map(|attr| attr.base_value))
            .collect::<Option<Vec<f64>>>()
            .and_then(|values| <[f64; 4]>::try_from(values).ok())
        else {
            continue;
        };
        let shift = module
            .attributes
            .get(&ATTR_RESISTANCE_SHIFT_AMOUNT)
            .map(|attr| attr.base_value / 100.0)
            .unwrap_or_default();

        // Weight the damage profile by the ship's armor resonances. At this
        // point only base values are available; this matches pyfa's caveat
        // that other resistance modules are not taken into account.
        let mut base_damage_taken = [0.0; 4];
        for i in 0..4 {
            let resonance = ship
                .hull
                .attributes
                .get(&ARMOR_RESONANCE[i])
                .map(|attr| attr.base_value)
                .unwrap_or(1.0);
            base_damage_taken[i] = damage[i] * resonance;
        }

        let equilibrium = simulate_adaptation(base_damage_taken, initial, shift);
        for (i, attribute_id) in ARMOR_RESONANCE.into_iter().enumerate() {
            module.set_attribute(attribute_id, equilibrium[i]);
        }
    }
}
