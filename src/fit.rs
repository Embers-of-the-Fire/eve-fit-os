use std::collections::HashMap;

use crate::provider::FitProvider;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Type {
    pub group_id: i32,
    pub category_id: i32,
    pub capacity: Option<f64>,
    pub mass: Option<f64>,
    pub radius: Option<f64>,
    pub volume: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeDogmaAttribute {
    pub attribute_id: i32,
    pub value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeDogmaEffect {
    pub effect_id: i32,
    pub is_default: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DogmaAttribute {
    pub default_value: f64,
    pub high_is_good: bool,
    pub stackable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DogmaEffectModifierInfoDomain {
    ItemID = 0,
    ShipID = 1,
    CharID = 2,
    OtherID = 3,
    StructureID = 4,
    Target = 5,
    TargetID = 6,
}

impl<T: Into<i32>> From<T> for DogmaEffectModifierInfoDomain {
    fn from(value: T) -> Self {
        match value.into() {
            0 => Self::ItemID,
            1 => Self::ShipID,
            2 => Self::CharID,
            3 => Self::OtherID,
            4 => Self::StructureID,
            5 => Self::Target,
            6 => Self::TargetID,
            _ => Self::ItemID,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DogmaEffectModifierInfoFunc {
    ItemModifier = 0,
    LocationGroupModifier = 1,
    LocationModifier = 2,
    LocationRequiredSkillModifier = 3,
    OwnerRequiredSkillModifier = 4,
    EffectStopper = 5,
}

impl<T: Into<i32>> From<T> for DogmaEffectModifierInfoFunc {
    fn from(value: T) -> Self {
        match value.into() {
            0 => Self::ItemModifier,
            1 => Self::LocationGroupModifier,
            2 => Self::LocationModifier,
            3 => Self::LocationRequiredSkillModifier,
            4 => Self::OwnerRequiredSkillModifier,
            5 => Self::EffectStopper,
            _ => Self::ItemModifier,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DogmaEffectModifierInfo {
    pub domain: DogmaEffectModifierInfoDomain,
    pub func: DogmaEffectModifierInfoFunc,
    pub modified_attribute_id: Option<i32>,
    pub modifying_attribute_id: Option<i32>,
    pub operation: Option<i32>,
    pub group_id: Option<i32>,
    pub skill_type_id: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DogmaEffect {
    pub effect_category: i32,
    pub modifier_info: Vec<DogmaEffectModifierInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemState {
    Passive,
    Online,
    Active,
    Overload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSlotType {
    High,
    Medium,
    Low,
    Rig,
    SubSystem,
    Service,
    TacticalMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemCharge {
    pub type_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemSlot {
    pub slot_type: ItemSlotType,
    pub index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemModule {
    pub type_id: i32,
    pub slot: ItemSlot,
    pub state: ItemState,
    pub charge: Option<ItemCharge>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemDrone {
    pub type_id: i32,
    pub state: ItemState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemImplant {
    pub type_id: i32,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemFit {
    pub ship_type_id: i32,
    pub modules: Vec<ItemModule>,
    pub drones: Vec<ItemDrone>,
    pub implants: Vec<ItemImplant>,
}

#[derive(Debug, Clone)]
pub struct FitContainer {
    pub fit: ItemFit,
    pub skills: HashMap<i32, u8>,
}

impl FitContainer {
    pub fn new(fit: ItemFit, skills: HashMap<i32, u8>) -> Self {
        Self { fit, skills }
    }
}

impl FitProvider for FitContainer {
    fn fit(&self) -> &self::ItemFit {
        &self.fit
    }

    fn skills(&self) -> &HashMap<i32, u8> {
        &self.skills
    }
}
