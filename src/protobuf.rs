use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use prost::Message;

use crate::fit;
use crate::provider::InfoProvider;

pub mod efos {
    include!(concat!(env!("OUT_DIR"), "/efos.rs"));
}

fn load_protobuf<T: Message + std::default::Default>(
    path: &Path,
    name: &str,
) -> anyhow::Result<T> {
    let mut file = File::open(path.join(name).with_extension("pb2"))?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;

    let obj = T::decode(buf.as_slice())?;
    Ok(obj)
}

#[derive(Debug, Clone)]
pub struct TypeDogmaItem {
    pub attributes: Vec<fit::TypeDogmaAttribute>,
    pub effects: Vec<fit::TypeDogmaEffect>,
}

pub struct Database {
    pub types: HashMap<i32, fit::Type>,
    pub type_dogma: HashMap<i32, TypeDogmaItem>,
    pub dogma_attributes: HashMap<i32, fit::DogmaAttribute>,
    pub dogma_effects: HashMap<i32, fit::DogmaEffect>,
}

impl Database {
    pub fn init(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let dogma_attr: efos::DogmaAttributes = load_protobuf(path, "dogmaAttributes")?;
        let dogma_eff: efos::DogmaEffects = load_protobuf(path, "dogmaEffects")?;
        let type_dogma: efos::TypeDogma = load_protobuf(path, "typeDogma")?;
        let types: efos::Types = load_protobuf(path, "types")?;

        Ok(Self {
            types: types
                .entries
                .into_iter()
                .map(|(k, v)| {
                    (k, fit::Type {
                        group_id: v.group_id,
                        category_id: v.category_id,
                        capacity: v.capacity,
                        mass: v.mass,
                        radius: v.radius,
                        volume: v.volume,
                    })
                })
                .collect(),
            type_dogma: type_dogma
                .entries
                .into_iter()
                .map(|(k, v)| {
                    (k, TypeDogmaItem {
                        attributes: v
                            .dogma_attributes
                            .into_iter()
                            .map(|a| fit::TypeDogmaAttribute {
                                attribute_id: a.attribute_id,
                                value: a.value,
                            })
                            .collect(),
                        effects: v
                            .dogma_effects
                            .into_iter()
                            .map(|e| fit::TypeDogmaEffect {
                                effect_id: e.effect_id,
                                is_default: e.is_default,
                            })
                            .collect(),
                    })
                })
                .collect(),
            dogma_attributes: dogma_attr
                .entries
                .into_iter()
                .map(|(k, v)| {
                    (k, fit::DogmaAttribute {
                        default_value: v.default_value,
                        high_is_good: v.high_is_good,
                        stackable: v.stackable,
                    })
                })
                .collect(),
            dogma_effects: dogma_eff
                .entries
                .into_iter()
                .map(|(k, v)| {
                    (k, fit::DogmaEffect {
                        discharge_attribute_id: v.discharge_attribute_id,
                        duration_attribute_id: v.duration_attribute_id,
                        effect_category: v.effect_category,
                        electronic_chance: v.electronic_chance,
                        is_assistance: v.is_assistance,
                        is_offensive: v.is_offensive,
                        is_warp_safe: v.is_warp_safe,
                        propulsion_chance: v.propulsion_chance,
                        range_chance: v.range_chance,
                        range_attribute_id: v.range_attribute_id,
                        falloff_attribute_id: v.falloff_attribute_id,
                        tracking_speed_attribute_id: v.tracking_speed_attribute_id,
                        fitting_usage_chance_attribute_id: v
                            .fitting_usage_chance_attribute_id,
                        resistance_attribute_id: v.resistance_attribute_id,
                        modifier_info: v
                            .modifier_info
                            .into_iter()
                            .map(|m| fit::DogmaEffectModifierInfo {
                                domain: m.domain.into(),
                                func: m.func.into(),
                                modified_attribute_id: m.modified_attribute_id,
                                modifying_attribute_id: m.modifying_attribute_id,
                                operation: m.operation,
                                group_id: m.group_id,
                                skill_type_id: m.skill_type_id,
                            })
                            .collect(),
                    })
                })
                .collect(),
        })
    }
}

impl InfoProvider for Database {
    fn get_dogma_attribute(&self, attribute_id: i32) -> crate::fit::DogmaAttribute {
        *self.dogma_attributes.get(&attribute_id).unwrap()
    }

    fn get_dogma_attributes(
        &self,
        type_id: i32,
    ) -> Vec<crate::fit::TypeDogmaAttribute> {
        self.type_dogma
            .get(&type_id)
            .map(|t| t.attributes.clone())
            .unwrap_or_default()
    }

    fn get_dogma_effect(&self, effect_id: i32) -> crate::fit::DogmaEffect {
        self.dogma_effects.get(&effect_id).unwrap().clone()
    }

    fn get_dogma_effects(&self, type_id: i32) -> Vec<fit::TypeDogmaEffect> {
        self.type_dogma
            .get(&type_id)
            .map(|t| t.effects.clone())
            .unwrap_or_default()
    }

    fn get_type(&self, type_id: i32) -> fit::Type {
        *self.types.get(&type_id).unwrap()
    }
}
