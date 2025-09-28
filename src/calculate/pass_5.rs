use std::collections::HashMap;

use super::Ship;
use super::item::{Attribute, EffectOperator, Item, Object};
use crate::calculate::item::{ModifierSource, ModifierTracker};
use crate::constant::PENALTY_FACTOR;
use crate::provider::{FitProvider, InfoProvider};

const OPERATOR_HAS_PENALTY: [EffectOperator; 5] = [
    EffectOperator::PreMul,
    EffectOperator::PostMul,
    EffectOperator::PostPercent,
    EffectOperator::PreDiv,
    EffectOperator::PostDiv,
];

#[derive(Default, Debug)]
pub(super) struct Cache {
    pub hull: HashMap<i32, f64>,
    pub character: HashMap<i32, f64>,
    pub structure: HashMap<i32, f64>,
    pub target: HashMap<i32, f64>,
    pub items: HashMap<usize, HashMap<i32, f64>>,
    pub implants: HashMap<usize, HashMap<i32, f64>>,
    pub boosters: HashMap<usize, HashMap<i32, f64>>,
    pub charge: HashMap<usize, HashMap<i32, f64>>,
    pub skills: HashMap<usize, HashMap<i32, f64>>,

    pub buffs: HashMap<i32, f64>,
}

impl Attribute {
    pub(super) fn calculate_value(
        &self,
        info: &impl InfoProvider,
        ship: &Ship,
        cache: &mut Cache,
        item: Object,
        attribute_id: i32,
    ) -> f64 {
        if let Some(value) = self.value {
            return value;
        }

        let cache_value = match item {
            Object::Ship => cache.hull.get(&attribute_id),
            Object::Character => cache.character.get(&attribute_id),
            Object::Structure => cache.structure.get(&attribute_id),
            Object::Target => cache.target.get(&attribute_id),
            Object::Item(index) => {
                cache.items.get(&index).and_then(|x| x.get(&attribute_id))
            }
            Object::Implant(index) => cache
                .implants
                .get(&index)
                .and_then(|x| x.get(&attribute_id)),
            Object::Booster(index) => {
                cache.charge.get(&index).and_then(|x| x.get(&attribute_id))
            }
            Object::Charge(index) => {
                cache.charge.get(&index).and_then(|x| x.get(&attribute_id))
            }
            Object::Skill(index) => {
                cache.skills.get(&index).and_then(|x| x.get(&attribute_id))
            }
        };

        if let Some(val) = cache_value {
            return *val;
        }

        let mut current_value = self.base_value;
        for operator in EffectOperator::iter() {
            let mut no_penalty = vec![];
            let mut penalty_positive = vec![];
            let mut penalty_negative = vec![];

            for effect in &self.effects {
                if effect.operator != operator {
                    continue;
                }

                let source = match effect.source {
                    Object::Ship => &ship.hull,
                    Object::Item(index) => &ship.modules[index],
                    Object::Implant(index) => &ship.implants[index],
                    Object::Booster(index) => &ship.boosters[index],
                    Object::Charge(index) => {
                        if let Some(charge) = &ship.modules[index].charge {
                            charge
                        } else {
                            continue;
                        }
                    }
                    Object::Skill(index) => &ship.skills[index],
                    Object::Character => &ship.character,
                    Object::Structure => &ship.structure,
                    Object::Target => &ship.target,
                };

                if effect.source_category > source.state {
                    continue;
                }

                let source_value = if let Some(attr) =
                    source.attributes.get(&effect.source_attribute_id)
                {
                    attr.calculate_value(
                        info,
                        ship,
                        cache,
                        effect.source,
                        effect.source_attribute_id,
                    )
                } else {
                    info.get_dogma_attribute(effect.source_attribute_id)
                        .default_value
                };
                let normalized_value = operator.into_func()(source_value);

                let value_group = (
                    normalized_value,
                    source_value,
                    ModifierSource::Effect(*effect),
                );

                if effect.penalty && OPERATOR_HAS_PENALTY.contains(&effect.operator) {
                    if normalized_value.is_sign_negative() {
                        penalty_negative.push(value_group);
                    } else {
                        penalty_positive.push(value_group);
                    }
                } else {
                    no_penalty.push(value_group);
                }
            }

            for buff_id in &self.buffs {
                let buff = info.get_buff(*buff_id);
                if operator != buff.operation.into() {
                    continue;
                }

                let source_value =
                    cache.buffs.get(buff_id).copied().unwrap_or_default();
                let normalized_value = operator.into_func()(source_value);

                let value_group = (
                    normalized_value,
                    source_value,
                    ModifierSource::Buff { buff_id: *buff_id },
                );
                if OPERATOR_HAS_PENALTY.contains(&buff.operation.into()) {
                    if normalized_value.is_sign_negative() {
                        penalty_negative.push(value_group);
                    } else {
                        penalty_positive.push(value_group);
                    }
                } else {
                    no_penalty.push(value_group);
                }
            }

            if no_penalty.is_empty()
                && penalty_positive.is_empty()
                && penalty_negative.is_empty()
            {
                continue;
            }

            match operator {
                EffectOperator::PreAssign | EffectOperator::PostAssign => {
                    let dogma_attribute = info.get_dogma_attribute(attribute_id);

                    let current_group = if dogma_attribute.high_is_good {
                        no_penalty
                            .iter()
                            .max_by(|(x, _, _), (y, _, _)| x.abs().total_cmp(&y.abs()))
                            .copied()
                    } else {
                        no_penalty
                            .iter()
                            .min_by(|(x, _, _), (y, _, _)| x.abs().total_cmp(&y.abs()))
                            .copied()
                    };

                    if let Some(group) = current_group {
                        debug_assert!(penalty_positive.is_empty());
                        debug_assert!(penalty_negative.is_empty());

                        current_value = group.0;
                        let tracker = ModifierTracker {
                            source: group.2,
                            original_value: group.1,
                            normalized_value: group.0,
                            penalized_value: group.0,
                        };
                        self.tracked_modifiers.borrow_mut().push(tracker);
                    }
                }

                EffectOperator::PreMul
                | EffectOperator::PreDiv
                | EffectOperator::PostMul
                | EffectOperator::PostDiv
                | EffectOperator::PostPercent => {
                    // no_penalty are non-stacking.
                    for group in no_penalty {
                        current_value *= 1.0 + group.0;
                        let tracker = ModifierTracker {
                            source: group.2,
                            original_value: group.1,
                            normalized_value: group.0,
                            penalized_value: group.0,
                        };
                        self.tracked_modifiers.borrow_mut().push(tracker);
                    }

                    // For positive values, the highest number goes first. For negative values, the lowest number.
                    penalty_positive
                        .sort_by(|(x, _, _), (y, _, _)| y.abs().total_cmp(&x.abs()));
                    penalty_negative
                        .sort_by(|(x, _, _), (y, _, _)| y.abs().total_cmp(&x.abs()));

                    // Apply positive stacking penalty.
                    for (index, group) in penalty_positive.iter().enumerate() {
                        let penalized_value =
                            group.0 * PENALTY_FACTOR.powi(index.pow(2) as i32);
                        current_value *= 1.0 + penalized_value;
                        let tracker = ModifierTracker {
                            source: group.2,
                            original_value: group.1,
                            normalized_value: group.0,
                            penalized_value,
                        };
                        self.tracked_modifiers.borrow_mut().push(tracker);
                    }
                    // Apply negative stacking penalty.
                    for (index, value) in penalty_negative.iter().enumerate() {
                        let penalized_value =
                            value.0 * PENALTY_FACTOR.powi(index.pow(2) as i32);
                        current_value *= 1.0 + penalized_value;
                        let tracker = ModifierTracker {
                            source: value.2,
                            original_value: value.1,
                            normalized_value: value.0,
                            penalized_value,
                        };
                        self.tracked_modifiers.borrow_mut().push(tracker);
                    }
                }

                EffectOperator::ModAdd | EffectOperator::ModSub => {
                    debug_assert!(penalty_positive.is_empty());
                    debug_assert!(penalty_negative.is_empty());

                    for value in no_penalty {
                        current_value += value.0;
                        let tracker = ModifierTracker {
                            source: value.2,
                            original_value: value.1,
                            normalized_value: value.0,
                            penalized_value: value.0,
                        };
                        self.tracked_modifiers.borrow_mut().push(tracker);
                    }
                }
            }
        }

        // special patch. ship resistance should be within 0.0 ~ 1.0.
        const RESONANCE_VALUE: &[i32] = &[
            271, 272, 273, 274, // shield
            267, 268, 269, 270, // armor
            113, 111, 109, 110, // hull
        ];
        let current_value = if RESONANCE_VALUE.contains(&attribute_id) {
            current_value.clamp(0.0, 1.0)
        } else {
            current_value
        };

        match item {
            Object::Ship => {
                cache.hull.insert(attribute_id, current_value);
            }
            Object::Character => {
                cache.character.insert(attribute_id, current_value);
            }
            Object::Structure => {
                cache.structure.insert(attribute_id, current_value);
            }
            Object::Target => {
                cache.target.insert(attribute_id, current_value);
            }
            Object::Item(index) => {
                cache
                    .items
                    .entry(index)
                    .or_default()
                    .insert(attribute_id, current_value);
            }
            Object::Implant(index) => {
                cache
                    .implants
                    .entry(index)
                    .or_default()
                    .insert(attribute_id, current_value);
            }
            Object::Booster(index) => {
                cache
                    .boosters
                    .entry(index)
                    .or_default()
                    .insert(attribute_id, current_value);
            }
            Object::Charge(index) => {
                cache
                    .charge
                    .entry(index)
                    .or_default()
                    .insert(attribute_id, current_value);
            }
            Object::Skill(index) => {
                cache
                    .skills
                    .entry(index)
                    .or_default()
                    .insert(attribute_id, current_value);
            }
        }

        current_value
    }
}

impl Item {
    fn calculate_values(
        &self,
        info: &impl InfoProvider,
        ship: &Ship,
        cache: &mut Cache,
        item: Object,
    ) {
        for (attribute_id, attribute) in self.attributes.iter() {
            attribute.calculate_value(info, ship, cache, item, *attribute_id);
        }
    }

    fn store_cached_values(
        &mut self,
        info: &impl InfoProvider,
        cache: &HashMap<i32, f64>,
    ) {
        for (attribute_id, value) in cache {
            if let Some(attribute) = self.attributes.get_mut(attribute_id) {
                attribute.value = Some(*value);
            } else {
                let dogma_attribute = info.get_dogma_attribute(*attribute_id);

                let mut attribute = Attribute::new_base(dogma_attribute.default_value);
                attribute.value = Some(*value);

                self.attributes.insert(*attribute_id, attribute);
            }
        }
    }
}

pub(crate) fn pass(
    _fit: &impl FitProvider,
    info: &impl InfoProvider,
    ship: &mut Ship,
    cache: &mut Cache,
) {
    ship.hull.calculate_values(info, ship, cache, Object::Ship);
    ship.character
        .calculate_values(info, ship, cache, Object::Character);
    ship.structure
        .calculate_values(info, ship, cache, Object::Structure);
    ship.target
        .calculate_values(info, ship, cache, Object::Target);
    for (index, item) in ship.modules.iter().enumerate() {
        item.calculate_values(info, ship, cache, Object::Item(index));
        if let Some(charge) = &item.charge {
            charge.calculate_values(info, ship, cache, Object::Charge(index));
        }
    }
    for (index, skill) in ship.skills.iter().enumerate() {
        skill.calculate_values(info, ship, cache, Object::Skill(index));
    }
    for (index, implant) in ship.implants.iter().enumerate() {
        implant.calculate_values(info, ship, cache, Object::Implant(index));
    }
    for (index, booster) in ship.boosters.iter().enumerate() {
        booster.calculate_values(info, ship, cache, Object::Booster(index));
    }

    ship.hull.store_cached_values(info, &cache.hull);
    ship.character.store_cached_values(info, &cache.character);
    ship.structure.store_cached_values(info, &cache.structure);
    ship.target.store_cached_values(info, &cache.target);
    for (index, item) in ship.modules.iter_mut().enumerate() {
        item.store_cached_values(info, &cache.items[&index]);
        if let Some(charge) = &mut item.charge {
            charge.store_cached_values(info, &cache.charge[&index]);
        }
    }
    for (index, skill) in ship.skills.iter_mut().enumerate() {
        skill.store_cached_values(info, &cache.skills[&index]);
    }
    for (index, implant) in ship.implants.iter_mut().enumerate() {
        implant.store_cached_values(info, &cache.implants[&index]);
    }
    for (index, booster) in ship.boosters.iter_mut().enumerate() {
        booster.store_cached_values(info, &cache.boosters[&index]);
    }
}
