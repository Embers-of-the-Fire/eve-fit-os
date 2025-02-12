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
        let dogma_effect: efos::DogmaEffects = load_protobuf(path, "dogmaEffects")?;
        let type_dogma: efos::TypeDogma = load_protobuf(path, "typeDogma")?;
        let types: efos::Types = load_protobuf(path, "types")?;

        Ok(Self::init_from_protobuf(
            dogma_attr,
            dogma_effect,
            type_dogma,
            types,
        ))
    }

    pub fn init_from_bytes(
        dogma_attr_buffer: &[u8],
        dogma_effect_buffer: &[u8],
        type_dogma_buffer: &[u8],
        types_buffer: &[u8],
    ) -> anyhow::Result<Self> {
        let dogma_attr: efos::DogmaAttributes =
            efos::DogmaAttributes::decode(dogma_attr_buffer)?;
        let dogma_effect: efos::DogmaEffects =
            efos::DogmaEffects::decode(dogma_effect_buffer)?;
        let type_dogma: efos::TypeDogma = efos::TypeDogma::decode(type_dogma_buffer)?;
        let types: efos::Types = efos::Types::decode(types_buffer)?;

        Ok(Self::init_from_protobuf(
            dogma_attr,
            dogma_effect,
            type_dogma,
            types,
        ))
    }

    pub fn init_from_protobuf(
        dogma_attr: efos::DogmaAttributes,
        dogma_effect: efos::DogmaEffects,
        type_dogma: efos::TypeDogma,
        types: efos::Types,
    ) -> Self {
        Self {
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
            dogma_effects: dogma_effect
                .entries
                .into_iter()
                .map(|(k, v)| {
                    (k, fit::DogmaEffect {
                        effect_category: v.effect_category,
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
        }
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
