use super::Ship;
use super::item::{Attribute, Item, Object};
use super::pass_5::Cache;
use crate::calculate::item::EffectCategory;
use crate::constant::ATTRIBUTE_SKILLS;
use crate::fit::{self, Buff};
use crate::provider::{FitProvider, InfoProvider};

/// Warfare buff attribute ID pairs `(buff_id, buff_value)` consulted by
/// pass 4. Also consumed by the platform fit-storage worker to prefetch the
/// referenced dogma attributes.
pub const WARFARE_BUFFS: [(i32, i32); 4] = [
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

    /// Registers a buff's `itemModifiers` on this item.
    ///
    /// `itemModifiers` describe attributes of the buff holder itself, so
    /// this must only be called on the ship hull. Injecting them into
    /// equipped items is wrong twice over: modules that carry the same
    /// attributes (e.g. shield resonances on a Damage Control) would forward
    /// the buffed value to the hull through their own dogma effects
    /// (double-dipping the buff and inflating EHP), and items that don't
    /// carry them (e.g. rigs) would gain spurious attributes.
    fn update_buff_holder(
        &mut self,
        info: &impl InfoProvider,
        buff_id: i32,
        buff: &Buff,
    ) {
        for m in buff.item_modifiers.iter().map(|u| u.dogma_attribute_id) {
            self.attributes
                .entry(m)
                .or_insert_with(|| {
                    Attribute::new_base(info.get_dogma_attribute(m).default_value)
                })
                .buffs
                .push(buff_id);
        }
    }

    /// Registers a buff's `location*Modifiers` on this item.
    ///
    /// Location modifiers describe items located on the buff holder, i.e.
    /// the ship's equipped items (modules, drones, fighters). They must
    /// never be registered on the hull itself.
    fn update_buff_location(
        &mut self,
        info: &impl InfoProvider,
        dynamic: &impl FitProvider,
        buff_id: i32,
        buff: &Buff,
    ) {
        let type_id = self.item_id.as_type_id(dynamic);
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
        // itemModifiers describe the buff holder itself (the ship hull,
        // e.g. shield resonances for the harmonizing charge).
        ship.hull.update_buff_holder(info, *buff_id, buff);
        // location*Modifiers describe items in the holder's location (fitted
        // modules, drones, fighters) — never the hull itself, and never
        // charges.
        for module in ship.modules.iter_mut() {
            module.update_buff_location(info, fit, *buff_id, buff);
        }
    }
}
