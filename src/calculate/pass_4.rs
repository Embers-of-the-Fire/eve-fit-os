use crate::{
    calculate::item::EffectCategory,
    constant::ATTRIBUTE_SKILLS,
    fit::{self, Buff},
    provider::{FitProvider, InfoProvider},
};

use super::{
    Ship,
    item::{Attribute, Item, Object},
    pass_5::Cache,
};

const WARFARE_BUFFS: [(i32, i32); 4] = [
    (2468, 2469), // warfareBuff1
    (2470, 2471), // warfareBuff2
    (2472, 2473), // warfareBuff3
    (2536, 2537), // warfareBuff4
];

impl Item {
    pub(super) fn calculate_warfares(
        &self,
        info: &impl InfoProvider,
        ship: &Ship,
        cache: &mut Cache,
        item: Object,
    ) {
        for (buff_id, buff_value) in WARFARE_BUFFS {
            let default_value =
                Attribute::new_base(info.get_dogma_attribute(buff_id).default_value);
            let buff_id_value_attr =
                self.attributes.get(&buff_id).unwrap_or(&default_value);
            let buff_id_value = buff_id_value_attr
                .calculate_value(info, ship, cache, item, buff_id)
                as i32;
            if buff_id_value == 0 {
                continue;
            }

            let default_value =
                Attribute::new_base(info.get_dogma_attribute(buff_value).default_value);
            let buff_value_value_attr =
                self.attributes.get(&buff_value).unwrap_or(&default_value);
            let buff_value_value = buff_value_value_attr
                .calculate_value(info, ship, cache, item, buff_value);

            if let Some(entry) = cache.buffs.get_mut(&buff_id_value) {
                let buff = info.get_buff(buff_id_value);
                match (buff.aggregate_mode, entry.total_cmp(&buff_value_value)) {
                    (fit::BuffAggregateMode::Maximum, std::cmp::Ordering::Less) => {
                        *entry = buff_value_value;
                    }
                    (fit::BuffAggregateMode::Minimum, std::cmp::Ordering::Greater) => {
                        *entry = buff_value_value;
                    }
                    _ => {}
                };
            } else {
                cache.buffs.insert(buff_id_value, buff_value_value);
            }
        }
    }

    fn update_buff(
        &mut self,
        info: &impl InfoProvider,
        dynamic: &impl FitProvider,
        buff_id: i32,
        buff: &Buff,
    ) {
        let type_id = self.item_id.as_type_id(dynamic);
        for m in buff.item_modifiers.iter().map(|u| u.dogma_attribute_id) {
            self.attributes
                .entry(m)
                .or_insert_with(|| {
                    Attribute::new_base(info.get_dogma_attribute(m).default_value)
                })
                .buffs
                .push(buff_id);
        }
        for m in buff.location_modifiers.iter().map(|u| u.dogma_attribute_id) {
            self.attributes
                .entry(m)
                .or_insert_with(|| {
                    Attribute::new_base(info.get_dogma_attribute(m).default_value)
                })
                .buffs
                .push(buff_id);
        }
        for m in &buff.location_group_modifiers {
            let ty = info.get_type(type_id);
            if ty.group_id == m.group_id {
                self.attributes
                    .entry(m.dogma_attribute_id)
                    .or_insert_with(|| {
                        Attribute::new_base(
                            info.get_dogma_attribute(m.dogma_attribute_id)
                                .default_value,
                        )
                    })
                    .buffs
                    .push(buff_id);
            }
        }
        for m in &buff.location_required_skill_modifiers {
            // Some skills apply on -1, indicating they should apply on anything that uses that skill.
            let skill_type_id = if m.skill_id == -1 {
                type_id
            } else {
                m.skill_id
            };

            for attribute_skill_id in &ATTRIBUTE_SKILLS {
                if self.attributes.contains_key(attribute_skill_id)
                    && self.attributes[attribute_skill_id].base_value
                        == skill_type_id as f64
                {
                    self.attributes
                        .entry(m.dogma_attribute_id)
                        .or_insert_with(|| {
                            Attribute::new_base(
                                info.get_dogma_attribute(m.dogma_attribute_id)
                                    .default_value,
                            )
                        })
                        .buffs
                        .push(buff_id);
                }
            }
        }
    }
}

pub(crate) fn pass(
    fit: &impl FitProvider,
    info: &impl InfoProvider,
    ship: &mut Ship,
    cache: &mut Cache,
) {
    for (index, item) in ship.modules.iter().enumerate() {
        if item.state >= EffectCategory::Active {
            item.calculate_warfares(info, ship, cache, Object::Item(index));
        }
    }
    for buff_id in cache.buffs.keys() {
        let buff = info.get_buff(*buff_id);
        for module in ship
            .modules
            .iter_mut()
            .chain(std::iter::once(&mut ship.hull))
        {
            module.update_buff(info, fit, *buff_id, buff);
            if let Some(charge) = &mut module.charge {
                charge.update_buff(info, fit, *buff_id, buff);
            }
        }
    }
}
