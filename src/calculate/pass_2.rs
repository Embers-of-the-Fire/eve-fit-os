use super::Ship;
use super::item::{Effect, EffectCategory, EffectOperator, Item, Object};
use crate::constant::{
    ATTRIBUTE_CAPACITOR_NEED_ID, ATTRIBUTE_SKILLS, CHARACTER_TYPE_ID,
    EXEMPT_PENALTY_CATEGORY_IDS,
};
use crate::fit::{DogmaEffectModifierInfoDomain, DogmaEffectModifierInfoFunc};
use crate::provider::{FitProvider, InfoProvider};

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modifier {
    LocationRequiredSkillModifier(i32),
    LocationGroupModifier(i32),
    LocationModifier,
    OwnerRequiredSkillModifier(i32),
    ItemModifier,
}

#[derive(Debug, Clone)]
pub(super) struct Pass2Effect {
    modifier: Modifier,
    operator: EffectOperator,
    source: Object,
    source_category: EffectCategory,
    source_attribute_id: i32,
    target: Object,
    target_attribute_id: i32,
}

fn get_modifier_func(
    func: DogmaEffectModifierInfoFunc,
    skill_type_id: Option<i32>,
    group_id: Option<i32>,
) -> Option<Modifier> {
    match func {
        DogmaEffectModifierInfoFunc::LocationRequiredSkillModifier => {
            Some(Modifier::LocationRequiredSkillModifier(skill_type_id?))
        }
        DogmaEffectModifierInfoFunc::LocationGroupModifier => {
            Some(Modifier::LocationGroupModifier(group_id?))
        }
        DogmaEffectModifierInfoFunc::LocationModifier => {
            Some(Modifier::LocationModifier)
        }
        DogmaEffectModifierInfoFunc::ItemModifier => Some(Modifier::ItemModifier),
        DogmaEffectModifierInfoFunc::OwnerRequiredSkillModifier => {
            Some(Modifier::OwnerRequiredSkillModifier(skill_type_id?))
        }
        // EffectStopper has no effect on the attributes; just on what you can bring online.
        DogmaEffectModifierInfoFunc::EffectStopper => None,
    }
}

/// Error(`None`): Invalid origin for `OtherID` domain.
fn get_target_object(domain: DogmaEffectModifierInfoDomain, origin: Object) -> Object {
    match domain {
        DogmaEffectModifierInfoDomain::ShipID => Object::Ship,
        DogmaEffectModifierInfoDomain::CharID => Object::Character,
        DogmaEffectModifierInfoDomain::OtherID => match origin {
            Object::Item(index) => Object::Charge(index),
            Object::Charge(index) => Object::Item(index),
            _ => panic!("Invalid origin for OtherID domain"),
        },
        DogmaEffectModifierInfoDomain::StructureID => Object::Structure,
        DogmaEffectModifierInfoDomain::ItemID => origin,
        DogmaEffectModifierInfoDomain::TargetID => Object::Target,
        DogmaEffectModifierInfoDomain::Target => Object::Target,
    }
}

fn get_effect_category(category: i32) -> EffectCategory {
    match category {
        0 => EffectCategory::Passive,
        1 => EffectCategory::Active,
        2 => EffectCategory::Target,
        3 => EffectCategory::Area,
        4 => EffectCategory::Online,
        5 => EffectCategory::Overload,
        6 => EffectCategory::Dungeon,
        7 => EffectCategory::System,
        _ => panic!("Unknown effect category: {}", category),
    }
}

fn get_effect_operator(operation: i32) -> Option<EffectOperator> {
    match operation {
        -1 => Some(EffectOperator::PreAssign),
        0 => Some(EffectOperator::PreMul),
        1 => Some(EffectOperator::PreDiv),
        2 => Some(EffectOperator::ModAdd),
        3 => Some(EffectOperator::ModSub),
        4 => Some(EffectOperator::PostMul),
        5 => Some(EffectOperator::PostDiv),
        6 => Some(EffectOperator::PostPercent),
        7 => Some(EffectOperator::PostAssign),
        // We ignore operator 9 (calculates Skill Level based on Skill Points; irrelevant for fits).
        9 => None,
        _ => panic!("Unknown effect operation: {}", operation),
    }
}

impl Item {
    pub(super) fn add_effect(
        &mut self,
        info: &impl InfoProvider,
        attribute_id: i32,
        source_category_id: i32,
        effect: &Pass2Effect,
    ) {
        let attr = info.get_dogma_attribute(attribute_id);

        if !self.attributes.contains_key(&attribute_id) {
            self.set_attribute(attribute_id, attr.default_value);
        }

        // Penalties are only count when an attribute is
        // not stackable and when the item is not in the exempt category.
        let penalty = !attr.stackable
            && !EXEMPT_PENALTY_CATEGORY_IDS.contains(&source_category_id);

        let attribute = self.attributes.get_mut(&attribute_id).unwrap();
        attribute.effects.push(Effect {
            operator: effect.operator,
            penalty,
            source: effect.source,
            source_category: effect.source_category,
            source_attribute_id: effect.source_attribute_id,
        });
    }

    fn collect_effects(
        &mut self,
        info: &impl InfoProvider,
        dynamic: &impl FitProvider,
        origin: Object,
        effects: &mut Vec<Pass2Effect>,
    ) {
        for dogma_effect in info.get_dogma_effects(self.item_id.as_type_id(dynamic)) {
            let type_dogma_effect = info.get_dogma_effect(dogma_effect.effect_id);
            let category = get_effect_category(type_dogma_effect.effect_category);

            if category > self.max_state && category <= EffectCategory::Overload {
                self.max_state = category;
            }

            if !type_dogma_effect.modifier_info.is_empty() {
                for modifier in type_dogma_effect.modifier_info {
                    let effect_modifier = get_modifier_func(
                        modifier.func,
                        modifier.skill_type_id,
                        modifier.group_id,
                    );
                    if effect_modifier.is_none() {
                        continue;
                    }

                    let operator = get_effect_operator(modifier.operation.unwrap());
                    if operator.is_none() {
                        continue;
                    }

                    // If the origin is an Item(), the domain is OtherID, but there is no charge, skip the effect.
                    if let (Object::Item(_), DogmaEffectModifierInfoDomain::OtherID) =
                        (&origin, &modifier.domain)
                    {
                        if self.charge.is_none() {
                            continue;
                        }
                    }

                    let target = get_target_object(modifier.domain, origin);
                    effects.push(Pass2Effect {
                        modifier: effect_modifier.unwrap(),
                        operator: operator.unwrap(),
                        source: origin,
                        source_category: category,
                        source_attribute_id: modifier.modifying_attribute_id.unwrap(),
                        target,
                        target_attribute_id: modifier.modified_attribute_id.unwrap(),
                    });
                }
            } else {
                self.effects.push(dogma_effect.effect_id);
            }
        }

        if self.attributes.contains_key(&ATTRIBUTE_CAPACITOR_NEED_ID)
            && self.max_state < EffectCategory::Active
        {
            self.max_state = EffectCategory::Active;
        }

        if self.state > self.max_state {
            self.state = self.max_state;
        }
    }
}

pub(super) fn pass(fit: &impl FitProvider, info: &impl InfoProvider, ship: &mut Ship) {
    let mut effects = vec![];

    ship.hull
        .collect_effects(info, fit, Object::Ship, &mut effects);
    ship.character
        .collect_effects(info, fit, Object::Character, &mut effects);
    for (index, item) in ship.modules.iter_mut().enumerate() {
        item.collect_effects(info, fit, Object::Item(index), &mut effects);
        if let Some(charge) = &mut item.charge {
            charge.collect_effects(info, fit, Object::Charge(index), &mut effects);
        }
    }
    for (index, skill) in ship.skills.iter_mut().enumerate() {
        skill.collect_effects(info, fit, Object::Skill(index), &mut effects);
    }
    for (index, implant) in ship.implants.iter_mut().enumerate() {
        implant.collect_effects(info, fit, Object::Implant(index), &mut effects);
    }

    for effect in effects {
        let source_type_id = match effect.source {
            Object::Ship => fit.fit().ship_type_id,
            Object::Item(index) => ship.modules[index].item_id.as_type_id(fit),
            Object::Implant(index) => ship.implants[index].item_id.as_type_id(fit),
            Object::Charge(index) => ship.modules[index]
                .charge
                .as_ref()
                .unwrap()
                .item_id
                .as_type_id(fit),
            Object::Skill(index) => ship.skills[index].item_id.as_type_id(fit),
            Object::Character => CHARACTER_TYPE_ID,
            Object::Structure => continue, // unimplemented
            Object::Target => continue,    // unimplemented
        };
        let category_id = info.get_type(source_type_id).category_id;

        match effect.modifier {
            Modifier::ItemModifier => {
                let target = match effect.target {
                    Object::Ship => &mut ship.hull,
                    Object::Character => &mut ship.character,
                    Object::Structure => &mut ship.structure,
                    Object::Item(index) => &mut ship.modules[index],
                    Object::Implant(index) => &mut ship.implants[index],
                    Object::Charge(index) => {
                        ship.modules[index].charge.as_mut().unwrap()
                    }
                    Object::Skill(index) => &mut ship.skills[index],
                    Object::Target => &mut ship.target,
                };
                target.add_effect(
                    info,
                    effect.target_attribute_id,
                    category_id,
                    &effect,
                );
            }
            Modifier::LocationModifier => {
                ship.hull.add_effect(
                    info,
                    effect.target_attribute_id,
                    category_id,
                    &effect,
                );

                for item in &mut ship.modules {
                    item.add_effect(
                        info,
                        effect.target_attribute_id,
                        category_id,
                        &effect,
                    );

                    if let Some(charge) = &mut item.charge {
                        charge.add_effect(
                            info,
                            effect.target_attribute_id,
                            category_id,
                            &effect,
                        );
                    }
                }
            }
            Modifier::LocationGroupModifier(group_id) => {
                let ty = info.get_type(ship.hull.item_id.as_type_id(fit));
                if ty.group_id == group_id {
                    ship.hull.add_effect(
                        info,
                        effect.target_attribute_id,
                        category_id,
                        &effect,
                    );
                }

                for item in &mut ship.modules {
                    let ty = info.get_type(item.item_id.as_type_id(fit));

                    if ty.group_id == group_id {
                        item.add_effect(
                            info,
                            effect.target_attribute_id,
                            category_id,
                            &effect,
                        );
                    }

                    if let Some(charge) = &mut item.charge {
                        let ty = info.get_type(charge.item_id.as_type_id(fit));

                        if ty.group_id == group_id {
                            charge.add_effect(
                                info,
                                effect.target_attribute_id,
                                category_id,
                                &effect,
                            );
                        }
                    }
                }
            }
            Modifier::OwnerRequiredSkillModifier(skill_type_id)
            | Modifier::LocationRequiredSkillModifier(skill_type_id) => {
                // Some skills apply on -1, indicating they should apply on anything that uses that skill.
                let skill_type_id = if skill_type_id == -1 {
                    source_type_id
                } else {
                    skill_type_id
                };

                for attribute_skill_id in &ATTRIBUTE_SKILLS {
                    if ship.hull.attributes.contains_key(attribute_skill_id)
                        && ship.hull.attributes[attribute_skill_id].base_value
                            == skill_type_id as f64
                    {
                        ship.hull.add_effect(
                            info,
                            effect.target_attribute_id,
                            category_id,
                            &effect,
                        );
                    }

                    for item in &mut ship.modules {
                        if item.attributes.contains_key(attribute_skill_id)
                            && item.attributes[attribute_skill_id].base_value
                                == skill_type_id as f64
                        {
                            item.add_effect(
                                info,
                                effect.target_attribute_id,
                                category_id,
                                &effect,
                            );
                        }

                        if let Some(charge) = &mut item.charge {
                            if charge.attributes.contains_key(attribute_skill_id)
                                && charge.attributes[attribute_skill_id].base_value
                                    == skill_type_id as f64
                            {
                                charge.add_effect(
                                    info,
                                    effect.target_attribute_id,
                                    category_id,
                                    &effect,
                                );
                            }
                        }
                    }

                    for item in &mut ship.implants {
                        if item.attributes.contains_key(attribute_skill_id)
                            && item.attributes[attribute_skill_id].base_value
                                == skill_type_id as f64
                        {
                            item.add_effect(
                                info,
                                effect.target_attribute_id,
                                category_id,
                                &effect,
                            );
                        }
                    }
                }
            }
        }
    }
}
