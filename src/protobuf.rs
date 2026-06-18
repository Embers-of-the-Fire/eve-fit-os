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

/// Load and decode a single protobuf message file at the given full path.
///
/// Opens the file, reads it into memory, and decodes it as `T`.
fn load_protobuf_file<T: Message + Default>(
    path: impl AsRef<Path>,
) -> anyhow::Result<T> {
    let mut file = File::open(path.as_ref())?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;
    Ok(T::decode(buf.as_slice())?)
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
    pub buff_collections: HashMap<i32, fit::Buff>,
}

impl Database {
    /// Initialize the database from a root directory.
    ///
    /// Expects files at `{root}/types.pb2`, `{root}/dogmaAttributes.pb2`,
    /// `{root}/dogmaEffects.pb2`, `{root}/typeDogma.pb2`, and
    /// `{root}/dbuffcollections.pb2`. Delegates to [`init_from_files`] after
    /// computing the five canonical file paths.
    pub fn init_from_root(root_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root = root_path.as_ref();
        Self::init_from_files(
            root.join("types").with_extension("pb2"),
            root.join("dogmaAttributes").with_extension("pb2"),
            root.join("dogmaEffects").with_extension("pb2"),
            root.join("typeDogma").with_extension("pb2"),
            root.join("dbuffcollections").with_extension("pb2"),
        )
    }

    /// Initialize the database from explicitly specified file paths.
    ///
    /// Each argument is the **full** path to the corresponding `.pb2` file on
    /// disk. No path joining is performed — the paths are used as-is. This is
    /// useful when the data files are scattered across directories or when the
    /// caller manages file resolution independently (e.g. via a content-addressed
    /// store).
    pub fn init_from_files(
        types: impl AsRef<Path>,
        dogma_attributes: impl AsRef<Path>,
        dogma_effects: impl AsRef<Path>,
        type_dogma: impl AsRef<Path>,
        buff_collections: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let dogma_attr: efos::DogmaAttributes = load_protobuf_file(dogma_attributes)?;
        let dogma_effect: efos::DogmaEffects = load_protobuf_file(dogma_effects)?;
        let type_dogma: efos::TypeDogma = load_protobuf_file(type_dogma)?;
        let types: efos::Types = load_protobuf_file(types)?;
        let buff_collections: efos::BuffCollections =
            load_protobuf_file(buff_collections)?;

        Ok(Self::init_from_protobuf(
            dogma_attr,
            dogma_effect,
            type_dogma,
            types,
            buff_collections,
        ))
    }

    pub fn init_from_bytes(
        dogma_attr_buffer: &[u8],
        dogma_effect_buffer: &[u8],
        type_dogma_buffer: &[u8],
        types_buffer: &[u8],
        buff_collections_buffer: &[u8],
    ) -> anyhow::Result<Self> {
        let dogma_attr: efos::DogmaAttributes =
            efos::DogmaAttributes::decode(dogma_attr_buffer)?;
        let dogma_effect: efos::DogmaEffects =
            efos::DogmaEffects::decode(dogma_effect_buffer)?;
        let type_dogma: efos::TypeDogma = efos::TypeDogma::decode(type_dogma_buffer)?;
        let types: efos::Types = efos::Types::decode(types_buffer)?;
        let buff_collections: efos::BuffCollections =
            efos::BuffCollections::decode(buff_collections_buffer)?;

        Ok(Self::init_from_protobuf(
            dogma_attr,
            dogma_effect,
            type_dogma,
            types,
            buff_collections,
        ))
    }

    pub fn init_from_protobuf(
        dogma_attr: efos::DogmaAttributes,
        dogma_effect: efos::DogmaEffects,
        type_dogma: efos::TypeDogma,
        types: efos::Types,
        buff_collections: efos::BuffCollections,
    ) -> Self {
        Self {
            types: types
                .entries
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        fit::Type {
                            group_id: v.group_id,
                            category_id: v.category_id,
                            capacity: v.capacity,
                            mass: v.mass,
                            radius: v.radius,
                            volume: v.volume,
                        },
                    )
                })
                .collect(),
            type_dogma: type_dogma
                .entries
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        TypeDogmaItem {
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
                        },
                    )
                })
                .collect(),
            dogma_attributes: dogma_attr
                .entries
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        fit::DogmaAttribute {
                            default_value: v.default_value,
                            high_is_good: v.high_is_good,
                            stackable: v.stackable,
                        },
                    )
                })
                .collect(),
            dogma_effects: dogma_effect
                .entries
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        fit::DogmaEffect {
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
                        },
                    )
                })
                .collect(),
            buff_collections: buff_collections
                .entries
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        fit::Buff {
                            aggregate_mode: match v.aggregate_mode {
                                0 => fit::BuffAggregateMode::Maximum,
                                1 => fit::BuffAggregateMode::Minimum,
                                _ => {
                                    panic!(
                                        "Unknown aggregate mode: {:?}",
                                        v.aggregate_mode
                                    )
                                }
                            },
                            operation: match v.operation_name {
                                0 => fit::BuffOperation::PreAssign,
                                1 => fit::BuffOperation::PreMul,
                                2 => fit::BuffOperation::PreDiv,
                                3 => fit::BuffOperation::ModAdd,
                                4 => fit::BuffOperation::ModSub,
                                5 => fit::BuffOperation::PostMul,
                                6 => fit::BuffOperation::PostDiv,
                                7 => fit::BuffOperation::PostPercent,
                                8 => fit::BuffOperation::PostAssign,
                                _ => {
                                    panic!(
                                        "Unknown buff operation: {:?}",
                                        v.operation_name
                                    )
                                }
                            },
                            item_modifiers: v
                                .item_modifiers
                                .into_iter()
                                .map(|m| fit::BuffItemModifier {
                                    dogma_attribute_id: m.dogma_attribute_id,
                                })
                                .collect(),
                            location_modifiers: v
                                .location_modifiers
                                .into_iter()
                                .map(|m| fit::BuffItemModifier {
                                    dogma_attribute_id: m.dogma_attribute_id,
                                })
                                .collect(),
                            location_group_modifiers: v
                                .location_group_modifiers
                                .into_iter()
                                .map(|m| fit::BuffGroupModifier {
                                    dogma_attribute_id: m.dogma_attribute_id,
                                    group_id: m.group_id,
                                })
                                .collect(),
                            location_required_skill_modifiers: v
                                .location_required_skill_modifiers
                                .into_iter()
                                .map(|m| fit::BuffSkillModifier {
                                    dogma_attribute_id: m.dogma_attribute_id,
                                    skill_id: m.skill_id,
                                })
                                .collect(),
                        },
                    )
                })
                .collect(),
        }
    }
}

impl InfoProvider for Database {
    fn get_dogma_attribute(&self, attribute_id: i32) -> &fit::DogmaAttribute {
        self.dogma_attributes.get(&attribute_id).unwrap()
    }

    fn get_dogma_attributes(&self, type_id: i32) -> Vec<fit::TypeDogmaAttribute> {
        self.type_dogma
            .get(&type_id)
            .map(|t| t.attributes.clone())
            .unwrap_or_default()
    }

    fn get_dogma_effect(&self, effect_id: i32) -> &fit::DogmaEffect {
        self.dogma_effects.get(&effect_id).unwrap()
    }

    fn get_dogma_effects(&self, type_id: i32) -> Vec<fit::TypeDogmaEffect> {
        self.type_dogma
            .get(&type_id)
            .map(|t| t.effects.clone())
            .unwrap_or_default()
    }

    fn get_buff(&self, buff_id: i32) -> &fit::Buff {
        self.buff_collections.get(&buff_id).unwrap()
    }

    fn get_type(&self, type_id: i32) -> &fit::Type {
        self.types.get(&type_id).unwrap()
    }
}
