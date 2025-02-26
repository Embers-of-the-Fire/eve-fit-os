use std::collections::HashMap;

use bitflags::bitflags;

use crate::constant::patches::attr::{
    ATTR_FIGHTER_ATTACK_MISSILE_ACTIVE, ATTR_FIGHTER_ATTACK_TURRET_ACTIVE,
    ATTR_FIGHTER_BOMB_ACTIVE, ATTR_FIGHTER_MISSILES_ACTIVE,
};
use crate::fit::{ItemSlotType, ItemState};
use crate::provider::FitProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EffectCategory {
    Passive,
    Online,
    Active,
    Overload,
    Target,
    Area,
    Dungeon,
    System,
}

impl From<ItemState> for EffectCategory {
    fn from(value: ItemState) -> Self {
        match value {
            ItemState::Passive => EffectCategory::Passive,
            ItemState::Online => EffectCategory::Online,
            ItemState::Active => EffectCategory::Active,
            ItemState::Overload => EffectCategory::Overload,
        }
    }
}

impl EffectCategory {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active | Self::Overload)
    }

    pub fn fallback_active(&self) -> Self {
        match self {
            Self::Passive => Self::Passive,
            _ => Self::Active,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectOperator {
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

impl EffectOperator {
    pub fn iter() -> impl IntoIterator<Item = Self, IntoIter: DoubleEndedIterator> {
        [
            Self::PreAssign,
            Self::PreMul,
            Self::PreDiv,
            Self::ModAdd,
            Self::ModSub,
            Self::PostMul,
            Self::PostDiv,
            Self::PostPercent,
            Self::PostAssign,
        ]
    }

    pub fn into_func(&self) -> fn(f64) -> f64 {
        match self {
            EffectOperator::PreAssign => |x| x,
            EffectOperator::PreMul => |x| x - 1.0,
            EffectOperator::PreDiv => |x| 1.0 / x - 1.0,
            EffectOperator::ModAdd => |x| x,
            EffectOperator::ModSub => |x| -x,
            EffectOperator::PostMul => |x| x - 1.0,
            EffectOperator::PostDiv => |x| 1.0 / x - 1.0,
            EffectOperator::PostPercent => |x| x / 100.0,
            EffectOperator::PostAssign => |x| x,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Object {
    Ship,
    Item(usize),
    Implant(usize),
    Charge(usize),
    Skill(usize),
    Character,
    Structure,
    Target,
}

#[derive(Debug, Clone, Copy)]
pub struct Effect {
    pub operator: EffectOperator,
    pub penalty: bool,
    pub source: Object,
    pub source_category: EffectCategory,
    pub source_attribute_id: i32,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub base_value: f64,
    pub value: Option<f64>,
    pub effects: Vec<Effect>,
}

impl Attribute {
    pub fn new_base(value: f64) -> Self {
        Self {
            base_value: value,
            value: None,
            effects: Vec::new(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FighterAbility: u8 {
        const ATTACK_TURRET = 0b0000_0001;
        const MISSILES = 0b0000_0010;
        const ATTACK_MISSILE = 0b0000_0100;
        const BOMB = 0b0000_1000;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SlotType {
    High,
    Medium,
    Low,
    Rig,
    SubSystem,
    Service,
    TacticalMode,
    DroneBay {
        group_id: u8,
    },
    Fighter {
        group_id: u8,
        ability: FighterAbility,
    },
    Charge,
    Implant,
    Fake,
}

impl From<ItemSlotType> for SlotType {
    fn from(value: ItemSlotType) -> Self {
        match value {
            ItemSlotType::High => SlotType::High,
            ItemSlotType::Medium => SlotType::Medium,
            ItemSlotType::Low => SlotType::Low,
            ItemSlotType::Rig => SlotType::Rig,
            ItemSlotType::SubSystem => SlotType::SubSystem,
            ItemSlotType::Service => SlotType::Service,
            ItemSlotType::TacticalMode => SlotType::TacticalMode,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Slot {
    pub slot_type: SlotType,
    pub index: Option<i32>,
}

impl Slot {
    pub fn is_module(&self) -> bool {
        matches!(
            self.slot_type,
            SlotType::High
                | SlotType::Medium
                | SlotType::Low
                | SlotType::Rig
                | SlotType::SubSystem
                | SlotType::TacticalMode
        )
    }
}

#[derive(Debug, Clone)]
pub struct Item {
    pub item_id: ItemID,
    pub slot: Slot,
    pub charge: Option<Box<Item>>,
    pub state: EffectCategory,
    pub max_state: EffectCategory,
    pub attributes: HashMap<i32, Attribute>,
    pub effects: Vec<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemID {
    Item(i32),
    Dynamic(i32),
}

impl ItemID {
    pub fn as_type_id(&self, dynamic: &impl FitProvider) -> i32 {
        match self {
            Self::Item(type_id) => *type_id,
            Self::Dynamic(dyn_id) => dynamic.get_dynamic_item_base_type_id(*dyn_id),
        }
    }
}

impl Item {
    // constructor

    pub fn new_charge(type_id: i32) -> Self {
        Self {
            item_id: ItemID::Item(type_id),
            slot: Slot {
                slot_type: SlotType::Charge,
                index: None,
            },
            charge: None,
            state: EffectCategory::Active,
            max_state: EffectCategory::Active,
            attributes: HashMap::new(),
            effects: Vec::new(),
        }
    }

    pub fn new_module(
        type_id: i32,
        slot: Slot,
        charge_type_id: Option<i32>,
        state: EffectCategory,
    ) -> Self {
        Self {
            item_id: ItemID::Item(type_id),
            slot,
            charge: charge_type_id
                .map(|charge_type_id| Box::new(Self::new_charge(charge_type_id))),
            state,
            max_state: EffectCategory::Passive,
            attributes: HashMap::new(),
            effects: Vec::new(),
        }
    }

    pub fn new_dynamic(
        item_id: i32,
        slot: Slot,
        charge_type_id: Option<i32>,
        state: EffectCategory,
    ) -> Self {
        Self {
            item_id: ItemID::Dynamic(item_id),
            slot,
            charge: charge_type_id
                .map(|charge_type_id| Box::new(Self::new_charge(charge_type_id))),
            state,
            max_state: EffectCategory::Passive,
            attributes: HashMap::new(),
            effects: Vec::new(),
        }
    }

    pub fn new_drone(type_id: i32, group_id: u8, state: EffectCategory) -> Self {
        Self {
            item_id: ItemID::Item(type_id),
            slot: Slot {
                slot_type: SlotType::DroneBay { group_id },
                index: None,
            },
            charge: None,
            state,
            max_state: EffectCategory::Active,
            attributes: HashMap::new(),
            effects: Vec::new(),
        }
    }

    pub fn new_fighter(
        type_id: i32,
        group_id: u8,
        state: EffectCategory,
        ability: FighterAbility,
    ) -> Self {
        let mut item = Self {
            item_id: ItemID::Item(type_id),
            slot: Slot {
                slot_type: SlotType::Fighter { group_id, ability },
                index: None,
            },
            charge: None,
            state,
            max_state: EffectCategory::Active,
            attributes: HashMap::new(),
            effects: Vec::new(),
        };

        if ability.contains(FighterAbility::ATTACK_TURRET) {
            item.add_attribute(ATTR_FIGHTER_ATTACK_TURRET_ACTIVE, 0.0, 1.0);
        }
        if ability.contains(FighterAbility::MISSILES) {
            item.add_attribute(ATTR_FIGHTER_MISSILES_ACTIVE, 0.0, 1.0);
        }
        if ability.contains(FighterAbility::ATTACK_MISSILE) {
            item.add_attribute(ATTR_FIGHTER_ATTACK_MISSILE_ACTIVE, 0.0, 1.0);
        }
        if ability.contains(FighterAbility::BOMB) {
            item.add_attribute(ATTR_FIGHTER_BOMB_ACTIVE, 0.0, 1.0);
        }

        item
    }

    pub fn new_implant(type_id: i32, index: i32) -> Self {
        Self {
            item_id: ItemID::Item(type_id),
            slot: Slot {
                slot_type: SlotType::Implant,
                index: Some(index),
            },
            charge: None,
            state: EffectCategory::Active,
            max_state: EffectCategory::Active,
            attributes: HashMap::new(),
            effects: Vec::new(),
        }
    }

    pub fn new_fake(type_id: i32) -> Self {
        Self {
            item_id: ItemID::Item(type_id),
            slot: Slot {
                slot_type: SlotType::Fake,
                index: None,
            },
            charge: None,
            state: EffectCategory::Active,
            max_state: EffectCategory::Active,
            attributes: HashMap::new(),
            effects: Vec::new(),
        }
    }

    pub fn new_tactical_mode(type_id: i32) -> Self {
        Self {
            item_id: ItemID::Item(type_id),
            slot: Slot {
                slot_type: SlotType::TacticalMode,
                index: None,
            },
            charge: None,
            state: EffectCategory::Active,
            max_state: EffectCategory::Active,
            attributes: HashMap::new(),
            effects: Vec::new(),
        }
    }
}
