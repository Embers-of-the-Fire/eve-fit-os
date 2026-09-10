use super::super::Ship;
use crate::calculate::item::{Attribute, Item, SlotType};
use crate::constant::patches::attr::{
    ATTR_DAMAGE_ALPHA, ATTR_DAMAGE_PER_SECOND_WITH_RELOAD,
    ATTR_DAMAGE_PER_SECOND_WITHOUT_RELOAD, ATTR_DAMAGE_VOLLEY,
};
use crate::constant::{
    ATTRIBUTE_DAMAGE_MULTIPLIER_BONUS_MAX_ID,
    ATTRIBUTE_DAMAGE_MULTIPLIER_BONUS_PER_CYCLE_ID, ATTRIBUTE_DAMAGE_MULTIPLIER_ID,
};
use crate::provider::FitProvider;

fn value(attribute: &Attribute) -> f64 {
    attribute.value.unwrap_or(attribute.base_value)
}

fn scale(attribute: &mut Attribute, factor: f64) {
    attribute.base_value *= factor;
    if let Some(value) = &mut attribute.value {
        *value *= factor;
    }
}

fn add_value(item: &mut Item, attribute_id: i32, delta: f64) {
    if let Some(attribute) = item.attributes.get_mut(&attribute_id) {
        if let Some(value) = &mut attribute.value {
            *value += delta;
        }
    }
}

/// Precursor turrets (Entropic Disintegrators) ramp up their damage with every
/// shot on the same target, gaining `damageMultiplierBonusPerCycle` per turn up
/// to `damageMultiplierBonusMax`. The spool is a runtime state that dogma does
/// not model, so the per-module `damage_turns` of the fit is applied here,
/// after all effects, such that ship and implant bonuses on the spool
/// attributes are already included.
///
/// The volley and DPS attributes are linear in the damage multiplier, so they
/// are rescaled in place and their deltas are added to the ship aggregates.
pub fn attribute_precursor_turret_spool(fit: &impl FitProvider, ship: &mut Ship) {
    let mut volley_delta = 0.0;
    let mut dps_delta = 0.0;
    let mut dps_reload_delta = 0.0;

    for fit_module in &fit.fit().modules {
        if fit_module.damage_turns == 0 {
            continue;
        }

        let slot_type = SlotType::from(fit_module.slot.slot_type);
        let Some(module) = ship.modules.iter_mut().find(|item| {
            item.slot.slot_type == slot_type
                && item.slot.index == Some(fit_module.slot.index)
        }) else {
            continue;
        };

        // A turret only spools while it is firing.
        if !module.state.is_active() {
            continue;
        }

        let per_cycle = module
            .attributes
            .get(&ATTRIBUTE_DAMAGE_MULTIPLIER_BONUS_PER_CYCLE_ID)
            .map(value)
            .unwrap_or_default();
        let max_bonus = module
            .attributes
            .get(&ATTRIBUTE_DAMAGE_MULTIPLIER_BONUS_MAX_ID)
            .map(value)
            .unwrap_or_default();
        if per_cycle <= 0.0 || max_bonus <= 0.0 {
            continue;
        }

        let factor = 1.0 + (fit_module.damage_turns as f64 * per_cycle).min(max_bonus);

        if let Some(attribute) =
            module.attributes.get_mut(&ATTRIBUTE_DAMAGE_MULTIPLIER_ID)
        {
            scale(attribute, factor);
        }
        if let Some(attribute) = module.attributes.get_mut(&ATTR_DAMAGE_VOLLEY) {
            volley_delta += value(attribute) * (factor - 1.0);
            scale(attribute, factor);
        }
        if let Some(attribute) = module
            .attributes
            .get_mut(&ATTR_DAMAGE_PER_SECOND_WITHOUT_RELOAD)
        {
            dps_delta += value(attribute) * (factor - 1.0);
            scale(attribute, factor);
        }
        if let Some(attribute) = module
            .attributes
            .get_mut(&ATTR_DAMAGE_PER_SECOND_WITH_RELOAD)
        {
            dps_reload_delta += value(attribute) * (factor - 1.0);
            scale(attribute, factor);
        }
    }

    add_value(&mut ship.hull, ATTR_DAMAGE_ALPHA, volley_delta);
    add_value(
        &mut ship.hull,
        ATTR_DAMAGE_PER_SECOND_WITHOUT_RELOAD,
        dps_delta,
    );
    add_value(
        &mut ship.hull,
        ATTR_DAMAGE_PER_SECOND_WITH_RELOAD,
        dps_reload_delta,
    );
}
