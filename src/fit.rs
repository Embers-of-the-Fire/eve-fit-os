use std::collections::HashMap;

use crate::calculate::DamageProfile;
use crate::calculate::item::{FighterAbility, ItemID};
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
pub struct BuffItemModifier {
    pub dogma_attribute_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuffGroupModifier {
    pub dogma_attribute_id: i32,
    pub group_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuffSkillModifier {
    pub dogma_attribute_id: i32,
    pub skill_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffAggregateMode {
    Maximum,
    Minimum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuffOperation {
    PreAssign,
    PreMul,
    PreDiv,
    ModAdd,
    ModSub,
    PostMul,
    PostDiv,
    PostPercent,
    PostAssign,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buff {
    pub aggregate_mode: BuffAggregateMode,
    pub item_modifiers: Vec<BuffItemModifier>,
    pub location_modifiers: Vec<BuffItemModifier>,
    pub location_group_modifiers: Vec<BuffGroupModifier>,
    pub location_required_skill_modifiers: Vec<BuffSkillModifier>,
    /// Modifiers applied to the charge of an equipped module when the
    /// charge requires the given skill. Only used by hand-authored data
    /// patches (system-wide effects); client data has no such modifiers.
    pub charge_required_skill_modifiers: Vec<BuffSkillModifier>,
    /// Whether this buff participates in stacking penalties.
    pub penalized: bool,
    pub operation: BuffOperation,
}

impl Buff {
    /// A buff with no modifiers: registering it on any attribute is a
    /// complete no-op. Used as the placeholder for buff IDs missing from the
    /// loaded data (e.g. stale snapshots or corrupt input), so missing buffs
    /// degrade silently instead of panicking.
    pub fn noop() -> Self {
        Self {
            aggregate_mode: BuffAggregateMode::Maximum,
            item_modifiers: Vec::new(),
            location_modifiers: Vec::new(),
            location_group_modifiers: Vec::new(),
            location_required_skill_modifiers: Vec::new(),
            charge_required_skill_modifiers: Vec::new(),
            penalized: true,
            operation: BuffOperation::PostAssign,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicItem {
    pub base_type: i32,
    /// attr key, factor
    pub dynamic_attributes: HashMap<i32, f64>,
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
    pub item_id: ItemID,
    pub slot: ItemSlot,
    pub state: ItemState,
    pub charge: Option<ItemCharge>,
    /// Number of completed damage ramp-up cycles of a precursor turret
    /// (0 = first shot, base damage).
    pub damage_turns: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemDrone {
    pub item_id: ItemID,
    pub group_id: u8,
    pub state: ItemState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemFighter {
    pub type_id: i32,
    pub group_id: u8,
    pub ability: FighterAbility,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemImplant {
    pub type_id: i32,
    pub index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemBooster {
    pub type_id: i32,
    pub index: i32,
}

/// A system-wide warfare buff (environmental effect) applied to the whole
/// fit, e.g. wormhole system effects, abyssal/metaliminal weather, or
/// sovereignty upgrades.
///
/// Unlike command bursts, these buffs are not sourced from a fitted module;
/// the `(buff_id, value)` pair is supplied directly by the user and merged
/// into the buff aggregation in calculate pass 4.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ItemSystemBuff {
    /// Buff collection ID (`dbuffcollections`).
    pub buff_id: i32,
    /// Buff strength, interpreted according to the buff's `operation`
    /// (e.g. percentage for `PostPercent`).
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemFit {
    pub ship_type_id: i32,
    pub damage_profile: DamageProfile,
    pub modules: Vec<ItemModule>,
    pub drones: Vec<ItemDrone>,
    pub fighters: Vec<ItemFighter>,
    pub implants: Vec<ItemImplant>,
    pub boosters: Vec<ItemBooster>,
    pub system_buffs: Vec<ItemSystemBuff>,
}

#[derive(Debug, Clone)]
pub struct FitContainer {
    pub fit: ItemFit,
    pub skills: HashMap<i32, u8>,
    pub dynamic: HashMap<i32, DynamicItem>,
}

impl FitContainer {
    pub fn new(
        fit: ItemFit,
        skills: HashMap<i32, u8>,
        dynamic: HashMap<i32, DynamicItem>,
    ) -> Self {
        Self {
            fit,
            skills,
            dynamic,
        }
    }
}

impl FitProvider for FitContainer {
    fn fit(&self) -> &self::ItemFit {
        &self.fit
    }

    fn skills(&self) -> &HashMap<i32, u8> {
        &self.skills
    }

    fn get_dynamic_item(&self, dynamic_item_id: i32) -> &DynamicItem {
        self.dynamic.get(&dynamic_item_id).unwrap()
    }

    fn get_dynamic_item_base_type_id(&self, dynamic_item_id: i32) -> i32 {
        self.dynamic.get(&dynamic_item_id).unwrap().base_type
    }
}
