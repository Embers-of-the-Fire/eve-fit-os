use std::collections::HashMap;

use crate::calculate::Ship;
use crate::calculate::item::{EffectCategory, Item, SlotType};
use crate::constant::patches::attr::{
    ATTR_CPU_FREE, ATTR_DRONE_ACTIVE, ATTR_DRONE_CAPACITY_LOAD, ATTR_POWER_FREE,
    ATTR_UPGRADE_USED,
};
use crate::fit::{FitContainer, ItemSlotType, ItemState, TypeDogmaAttribute};
use crate::provider::InfoProvider;

const EFFECT_LAUNCHER: i32 = 40;
const EFFECT_TURRET: i32 = 42;

const ATTR_LAUNCHER: i32 = 101;
const ATTR_TURRET: i32 = 102;
const ATTR_CHARGE_SIZE: i32 = 128;
const ATTR_CHARGE_RATE: i32 = 56;
const ATTR_VOLUME: i32 = 161;
const ATTR_AMMO_CAPACITY: i32 = 38;
const ATTR_MAX_ACTIVE: i32 = 763;
const ATTR_BOOSTER_SLOT: i32 = 1087;
const ATTR_SUBSYSTEM_TURRET: i32 = 1368;
const ATTR_SUBSYSTEM_LAUNCHER: i32 = 1369;
const ATTR_RIG_SIZE: i32 = 1547;

const ATTR_POWER_OUTPUT: i32 = 11;
const ATTR_CPU_OUTPUT: i32 = 48;
const ATTR_DRONE_CAPACITY: i32 = 283;
const ATTR_MAX_ACTIVE_DRONES: i32 = 352;
const ATTR_UPGRADE_CAPACITY: i32 = 1132;
const ATTR_DRONE_BANDWIDTH: i32 = 1271;
const ATTR_DRONE_BANDWIDTH_LOAD: i32 = 1273;
const ATTR_FIGHTER_TUBES: i32 = 2216;
const ATTR_FIGHTER_LIGHT_SLOTS: i32 = 2217;
const ATTR_FIGHTER_SUPPORT_SLOTS: i32 = 2218;
const ATTR_FIGHTER_HEAVY_SLOTS: i32 = 2219;

const GROUP_LIGHT_FIGHTER: [i32; 2] = [1652, 4777];
const GROUP_SUPPORT_FIGHTER: [i32; 2] = [1537, 4778];
const GROUP_HEAVY_FIGHTER: [i32; 2] = [1653, 4779];

const RESOURCE_EPSILON: f64 = 1e-6;

const ATTR_CHARGE_GROUPS: [i32; 5] = [604, 605, 606, 609, 610];
const CAN_FIT_GROUP_ATTR_IDS: [i32; 20] = [
    1298, 1299, 1300, 1301, 1872, 1879, 1880, 1881, 2065, 2396, 2476, 2477, 2478, 2479,
    2480, 2481, 2482, 2483, 2484, 2485,
];
const CAN_FIT_TYPE_ATTR_IDS: [i32; 12] = [
    1302, 1303, 1304, 1305, 1944, 2103, 2463, 2486, 2487, 2488, 2758, 5948,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSlotType {
    High,
    Medium,
    Low,
    Rig,
    SubSystem,
    Service,
    TacticalMode,
    Implant,
    Booster,
    Drone,
    Fighter,
    Ship,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FighterSquadron {
    Light,
    Support,
    Heavy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationState {
    Passive,
    Online,
    Active,
    Overload,
}

impl From<ItemState> for ValidationState {
    fn from(value: ItemState) -> Self {
        match value {
            ItemState::Passive => Self::Passive,
            ItemState::Online => Self::Online,
            ItemState::Active => Self::Active,
            ItemState::Overload => Self::Overload,
        }
    }
}

impl From<EffectCategory> for ValidationState {
    fn from(value: EffectCategory) -> Self {
        match value {
            EffectCategory::Passive => Self::Passive,
            EffectCategory::Online => Self::Online,
            EffectCategory::Active => Self::Active,
            _ => Self::Overload,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationIssue {
    pub slot_type: ValidationSlotType,
    pub index: Option<i32>,
    pub kind: ValidationIssueKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationIssueKind {
    Error(ValidationErrorKey),
    Warning(ValidationWarningKey),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationErrorKey {
    IncompatibleChargeSize {
        expected: u8,
        actual: u8,
    },
    IncompatibleChargeCapacity {
        max: f64,
        actual: f64,
    },
    IncompatibleChargeGroup {
        expected: Vec<i32>,
        actual: i32,
    },
    TooMuchTurret {
        expected: u8,
        actual: u8,
    },
    TooMuchLauncher {
        expected: u8,
        actual: u8,
    },
    ConflictItem {
        group_id: i32,
    },
    DuplicateBooster {
        slot: i32,
    },
    IncompatibleShipGroup {
        expected: Vec<i32>,
    },
    IncompatibleShipType {
        expected: Vec<i32>,
    },
    IncompatibleRigSize {
        expected: u8,
        actual: u8,
    },
    PowergridExceeded {
        expected: f64,
        actual: f64,
    },
    CpuExceeded {
        expected: f64,
        actual: f64,
    },
    CalibrationExceeded {
        expected: f64,
        actual: f64,
    },
    DroneBandwidthExceeded {
        expected: f64,
        actual: f64,
    },
    DroneBayExceeded {
        expected: f64,
        actual: f64,
    },
    TooManyActiveDrones {
        expected: u32,
        actual: u32,
    },
    TooMuchFighterTube {
        expected: u32,
        actual: u32,
    },
    TooMuchFighterSquadron {
        category: FighterSquadron,
        expected: u32,
        actual: u32,
    },
    StateExceedsMax {
        state: ValidationState,
        max_state: ValidationState,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationWarningKey {
    MissingCharge,
}

struct ValidationContext<'a> {
    fit: &'a FitContainer,
    ship: &'a Ship,
    info: &'a dyn InfoProvider,
}

type ValidationRule = fn(&ValidationContext<'_>, &mut Vec<ValidationIssue>);

const VALIDATION_RULES: &[ValidationRule] = &[
    validate_slot_counts,
    validate_fit_targets,
    validate_rig_sizes,
    validate_booster_slots,
    validate_charges,
    validate_max_active_groups,
    validate_ship_resources,
    validate_drone_capacity,
    validate_fighter_capacity,
    validate_module_states,
];

pub fn validate_fit(
    fit: &FitContainer,
    ship: &Ship,
    info: &impl InfoProvider,
) -> Vec<ValidationIssue> {
    let context = ValidationContext { fit, ship, info };
    let mut issues = Vec::new();
    for rule in VALIDATION_RULES {
        rule(&context, &mut issues);
    }
    issues
}

fn validate_slot_counts(
    context: &ValidationContext<'_>,
    issues: &mut Vec<ValidationIssue>,
) {
    let fit = context.fit;
    let (actual_turret, actual_launcher) = fit
        .fit
        .modules
        .iter()
        .filter(|module| matches!(module.slot.slot_type, ItemSlotType::High))
        .map(|module| {
            let type_id = module.item_id.as_type_id(fit);
            if context
                .info
                .get_dogma_effects(type_id)
                .iter()
                .any(|effect| effect.effect_id == EFFECT_TURRET)
            {
                (1, 0)
            } else if context
                .info
                .get_dogma_effects(type_id)
                .iter()
                .any(|effect| effect.effect_id == EFFECT_LAUNCHER)
            {
                (0, 1)
            } else {
                (0, 0)
            }
        })
        .fold(
            (0, 0),
            |(turret, launcher), (next_turret, next_launcher)| {
                (turret + next_turret, launcher + next_launcher)
            },
        );

    let ship_attributes = context.info.get_dogma_attributes(fit.fit.ship_type_id);
    let mut turret = find_attr(&ship_attributes, ATTR_TURRET).unwrap_or(0.0) as u8;
    let mut launcher = find_attr(&ship_attributes, ATTR_LAUNCHER).unwrap_or(0.0) as u8;

    for module in fit
        .fit
        .modules
        .iter()
        .filter(|module| matches!(module.slot.slot_type, ItemSlotType::SubSystem))
    {
        let type_id = module.item_id.as_type_id(fit);
        let attributes = context.info.get_dogma_attributes(type_id);
        turret += find_attr(&attributes, ATTR_SUBSYSTEM_TURRET).unwrap_or(0.0) as u8;
        launcher +=
            find_attr(&attributes, ATTR_SUBSYSTEM_LAUNCHER).unwrap_or(0.0) as u8;
    }

    if actual_turret > turret {
        issues.push(ValidationIssue {
            slot_type: ValidationSlotType::High,
            index: None,
            kind: ValidationIssueKind::Error(ValidationErrorKey::TooMuchTurret {
                expected: turret,
                actual: actual_turret,
            }),
        });
    }
    if actual_launcher > launcher {
        issues.push(ValidationIssue {
            slot_type: ValidationSlotType::High,
            index: None,
            kind: ValidationIssueKind::Error(ValidationErrorKey::TooMuchLauncher {
                expected: launcher,
                actual: actual_launcher,
            }),
        });
    }
}

fn validate_fit_targets(
    context: &ValidationContext<'_>,
    issues: &mut Vec<ValidationIssue>,
) {
    let fit = context.fit;
    let ship_type = context.info.get_type(fit.fit.ship_type_id);

    for module in fit.fit.modules.iter() {
        let Some(slot_type) = validation_slot_type(module.slot.slot_type) else {
            continue;
        };
        let type_id = module.item_id.as_type_id(fit);
        let groups = context
            .info
            .get_dogma_attributes(type_id)
            .into_iter()
            .filter(|attr| CAN_FIT_GROUP_ATTR_IDS.contains(&attr.attribute_id))
            .map(|attr| attr.value as i32)
            .filter(|&group_id| group_id != 0)
            .collect::<Vec<_>>();
        let types = context
            .info
            .get_dogma_attributes(type_id)
            .into_iter()
            .filter(|attr| CAN_FIT_TYPE_ATTR_IDS.contains(&attr.attribute_id))
            .map(|attr| attr.value as i32)
            .filter(|&type_id| type_id != 0)
            .collect::<Vec<_>>();

        let group_matches = groups.contains(&ship_type.group_id);
        let type_matches = types.contains(&fit.fit.ship_type_id);
        if group_matches || type_matches || (groups.is_empty() && types.is_empty()) {
            continue;
        }

        if !groups.is_empty() {
            issues.push(ValidationIssue {
                slot_type,
                index: Some(module.slot.index),
                kind: ValidationIssueKind::Error(
                    ValidationErrorKey::IncompatibleShipGroup { expected: groups },
                ),
            });
        }
        if !types.is_empty() {
            issues.push(ValidationIssue {
                slot_type,
                index: Some(module.slot.index),
                kind: ValidationIssueKind::Error(
                    ValidationErrorKey::IncompatibleShipType { expected: types },
                ),
            });
        }
    }
}

fn validate_rig_sizes(
    context: &ValidationContext<'_>,
    issues: &mut Vec<ValidationIssue>,
) {
    let fit = context.fit;
    let ship_rig_size = context
        .info
        .get_dogma_attributes(fit.fit.ship_type_id)
        .iter()
        .find_map(|attr| {
            (attr.attribute_id == ATTR_RIG_SIZE).then_some(attr.value as u8)
        });

    for module in fit
        .fit
        .modules
        .iter()
        .filter(|module| matches!(module.slot.slot_type, ItemSlotType::Rig))
    {
        let rig_size = context
            .info
            .get_dogma_attributes(module.item_id.as_type_id(fit))
            .iter()
            .find_map(|attr| {
                (attr.attribute_id == ATTR_RIG_SIZE).then_some(attr.value as u8)
            });

        if let (Some(expected), Some(actual)) = (ship_rig_size, rig_size) {
            if expected != actual {
                issues.push(ValidationIssue {
                    slot_type: ValidationSlotType::Rig,
                    index: Some(module.slot.index),
                    kind: ValidationIssueKind::Error(
                        ValidationErrorKey::IncompatibleRigSize { expected, actual },
                    ),
                });
            }
        }
    }
}

fn validate_booster_slots(
    context: &ValidationContext<'_>,
    issues: &mut Vec<ValidationIssue>,
) {
    let fit = context.fit;
    let mut slots: HashMap<i32, usize> = HashMap::new();
    for booster in &fit.fit.boosters {
        if let Some(slot) = booster_slot(booster.type_id, context.info) {
            *slots.entry(slot).or_default() += 1;
        }
    }

    for booster in &fit.fit.boosters {
        if let Some(slot) = booster_slot(booster.type_id, context.info) {
            if slots.get(&slot).is_some_and(|&count| count > 1) {
                issues.push(ValidationIssue {
                    slot_type: ValidationSlotType::Booster,
                    index: Some(booster.index),
                    kind: ValidationIssueKind::Error(
                        ValidationErrorKey::DuplicateBooster { slot },
                    ),
                });
            }
        }
    }
}

fn validate_charges(
    context: &ValidationContext<'_>,
    issues: &mut Vec<ValidationIssue>,
) {
    for item in context
        .ship
        .modules
        .iter()
        .filter(|item| is_primary_module_slot(item))
    {
        let Some(slot_type) = output_validation_slot_type(item) else {
            continue;
        };
        let ammo_capacity = item_attribute(item, ATTR_AMMO_CAPACITY);
        if let Some(charge) = &item.charge {
            if let (Some(max), Some(actual)) =
                (ammo_capacity, item_attribute(charge, ATTR_VOLUME))
            {
                if actual > max {
                    issues.push(ValidationIssue {
                        slot_type,
                        index: item.slot.index,
                        kind: ValidationIssueKind::Error(
                            ValidationErrorKey::IncompatibleChargeCapacity {
                                max,
                                actual,
                            },
                        ),
                    });
                }
            }

            if let Some(expected) = item_attribute(item, ATTR_CHARGE_SIZE) {
                if let Some(actual) = item_attribute(charge, ATTR_CHARGE_SIZE) {
                    if expected as u8 != actual as u8 {
                        issues.push(ValidationIssue {
                            slot_type,
                            index: item.slot.index,
                            kind: ValidationIssueKind::Error(
                                ValidationErrorKey::IncompatibleChargeSize {
                                    expected: expected as u8,
                                    actual: actual as u8,
                                },
                            ),
                        });
                    }
                }
            }

            let expected_groups = item_accepted_charge_groups(item);
            if !expected_groups.is_empty() {
                if let Some(actual) = item_group_id(context, charge) {
                    if !expected_groups.contains(&actual) {
                        issues.push(ValidationIssue {
                            slot_type,
                            index: item.slot.index,
                            kind: ValidationIssueKind::Error(
                                ValidationErrorKey::IncompatibleChargeGroup {
                                    expected: expected_groups,
                                    actual,
                                },
                            ),
                        });
                    }
                }
            }
        } else if ammo_capacity.is_some() && item_consumes_charge(item) {
            issues.push(ValidationIssue {
                slot_type,
                index: item.slot.index,
                kind: ValidationIssueKind::Warning(ValidationWarningKey::MissingCharge),
            });
        }
    }
}

fn validate_max_active_groups(
    context: &ValidationContext<'_>,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut counts: HashMap<i32, usize> = HashMap::new();
    let mut limits: HashMap<i32, usize> = HashMap::new();
    for item in context
        .ship
        .modules
        .iter()
        .filter(|item| is_primary_module_slot(item) && item.state.is_active())
    {
        let Some(limit) = max_active_limit(item) else {
            continue;
        };
        let Some(group_id) = item_group_id(context, item) else {
            continue;
        };
        *counts.entry(group_id).or_default() += 1;
        limits
            .entry(group_id)
            .and_modify(|current| *current = (*current).min(limit))
            .or_insert(limit);
    }

    counts.retain(|group_id, count| {
        limits.get(group_id).is_some_and(|limit| *count > *limit)
    });

    for item in context
        .ship
        .modules
        .iter()
        .filter(|item| is_primary_module_slot(item) && item.state.is_active())
    {
        if max_active_limit(item).is_none() {
            continue;
        }
        let Some(group_id) = item_group_id(context, item) else {
            continue;
        };
        if !counts.contains_key(&group_id) {
            continue;
        }
        let Some(slot_type) = output_validation_slot_type(item) else {
            continue;
        };
        issues.push(ValidationIssue {
            slot_type,
            index: item.slot.index,
            kind: ValidationIssueKind::Error(ValidationErrorKey::ConflictItem {
                group_id,
            }),
        });
    }
}

fn validate_ship_resources(
    context: &ValidationContext<'_>,
    issues: &mut Vec<ValidationIssue>,
) {
    let hull = &context.ship.hull;

    if let Some(free) = item_attribute(hull, ATTR_POWER_FREE) {
        if free < -RESOURCE_EPSILON {
            let expected = item_attribute(hull, ATTR_POWER_OUTPUT).unwrap_or(0.0);
            issues.push(ValidationIssue {
                slot_type: ValidationSlotType::Ship,
                index: None,
                kind: ValidationIssueKind::Error(
                    ValidationErrorKey::PowergridExceeded {
                        expected,
                        actual: expected - free,
                    },
                ),
            });
        }
    }

    if let Some(free) = item_attribute(hull, ATTR_CPU_FREE) {
        if free < -RESOURCE_EPSILON {
            let expected = item_attribute(hull, ATTR_CPU_OUTPUT).unwrap_or(0.0);
            issues.push(ValidationIssue {
                slot_type: ValidationSlotType::Ship,
                index: None,
                kind: ValidationIssueKind::Error(ValidationErrorKey::CpuExceeded {
                    expected,
                    actual: expected - free,
                }),
            });
        }
    }

    let used = item_attribute(hull, ATTR_UPGRADE_USED).unwrap_or(0.0);
    let capacity = item_attribute(hull, ATTR_UPGRADE_CAPACITY).unwrap_or(0.0);
    if used > capacity + RESOURCE_EPSILON {
        issues.push(ValidationIssue {
            slot_type: ValidationSlotType::Ship,
            index: None,
            kind: ValidationIssueKind::Error(ValidationErrorKey::CalibrationExceeded {
                expected: capacity,
                actual: used,
            }),
        });
    }
}

fn validate_drone_capacity(
    context: &ValidationContext<'_>,
    issues: &mut Vec<ValidationIssue>,
) {
    let hull = &context.ship.hull;

    let bandwidth_load = item_attribute(hull, ATTR_DRONE_BANDWIDTH_LOAD).unwrap_or(0.0);
    let bandwidth = item_attribute(hull, ATTR_DRONE_BANDWIDTH).unwrap_or(0.0);
    if bandwidth_load > bandwidth + RESOURCE_EPSILON {
        issues.push(ValidationIssue {
            slot_type: ValidationSlotType::Drone,
            index: None,
            kind: ValidationIssueKind::Error(
                ValidationErrorKey::DroneBandwidthExceeded {
                    expected: bandwidth,
                    actual: bandwidth_load,
                },
            ),
        });
    }

    let bay_load = item_attribute(hull, ATTR_DRONE_CAPACITY_LOAD).unwrap_or(0.0);
    let bay = item_attribute(hull, ATTR_DRONE_CAPACITY).unwrap_or(0.0);
    if bay_load > bay + RESOURCE_EPSILON {
        issues.push(ValidationIssue {
            slot_type: ValidationSlotType::Drone,
            index: None,
            kind: ValidationIssueKind::Error(ValidationErrorKey::DroneBayExceeded {
                expected: bay,
                actual: bay_load,
            }),
        });
    }

    let active = item_attribute(hull, ATTR_DRONE_ACTIVE).unwrap_or(0.0);
    if let Some(max_active) =
        item_attribute(&context.ship.character, ATTR_MAX_ACTIVE_DRONES)
    {
        if active > max_active + RESOURCE_EPSILON {
            issues.push(ValidationIssue {
                slot_type: ValidationSlotType::Drone,
                index: None,
                kind: ValidationIssueKind::Error(
                    ValidationErrorKey::TooManyActiveDrones {
                        expected: max_active as u32,
                        actual: active as u32,
                    },
                ),
            });
        }
    }
}

fn validate_fighter_capacity(
    context: &ValidationContext<'_>,
    issues: &mut Vec<ValidationIssue>,
) {
    if context.fit.fit.fighters.is_empty() {
        return;
    }
    let hull = &context.ship.hull;

    let total = context.fit.fit.fighters.len() as u32;
    let tubes = item_attribute(hull, ATTR_FIGHTER_TUBES).unwrap_or(0.0) as u32;
    if total > tubes {
        issues.push(ValidationIssue {
            slot_type: ValidationSlotType::Fighter,
            index: None,
            kind: ValidationIssueKind::Error(ValidationErrorKey::TooMuchFighterTube {
                expected: tubes,
                actual: total,
            }),
        });
    }

    for (category, groups, limit_attr) in [
        (
            FighterSquadron::Light,
            GROUP_LIGHT_FIGHTER,
            ATTR_FIGHTER_LIGHT_SLOTS,
        ),
        (
            FighterSquadron::Support,
            GROUP_SUPPORT_FIGHTER,
            ATTR_FIGHTER_SUPPORT_SLOTS,
        ),
        (
            FighterSquadron::Heavy,
            GROUP_HEAVY_FIGHTER,
            ATTR_FIGHTER_HEAVY_SLOTS,
        ),
    ] {
        let count = context
            .fit
            .fit
            .fighters
            .iter()
            .filter(|fighter| {
                groups.contains(&context.info.get_type(fighter.type_id).group_id)
            })
            .count() as u32;
        if count == 0 {
            continue;
        }
        let limit = item_attribute(hull, limit_attr).unwrap_or(0.0) as u32;
        if count > limit {
            issues.push(ValidationIssue {
                slot_type: ValidationSlotType::Fighter,
                index: None,
                kind: ValidationIssueKind::Error(
                    ValidationErrorKey::TooMuchFighterSquadron {
                        category,
                        expected: limit,
                        actual: count,
                    },
                ),
            });
        }
    }
}

fn validate_module_states(
    context: &ValidationContext<'_>,
    issues: &mut Vec<ValidationIssue>,
) {
    // pass_1 pushes fitted modules first, preserving input order.
    for (module, item) in context
        .fit
        .fit
        .modules
        .iter()
        .zip(context.ship.modules.iter())
    {
        let state = EffectCategory::from(module.state);
        if state <= item.max_state {
            continue;
        }
        let Some(slot_type) = validation_slot_type(module.slot.slot_type) else {
            continue;
        };
        issues.push(ValidationIssue {
            slot_type,
            index: Some(module.slot.index),
            kind: ValidationIssueKind::Error(ValidationErrorKey::StateExceedsMax {
                state: module.state.into(),
                max_state: item.max_state.into(),
            }),
        });
    }
}

fn validation_slot_type(slot_type: ItemSlotType) -> Option<ValidationSlotType> {
    match slot_type {
        ItemSlotType::High => Some(ValidationSlotType::High),
        ItemSlotType::Medium => Some(ValidationSlotType::Medium),
        ItemSlotType::Low => Some(ValidationSlotType::Low),
        ItemSlotType::Rig => Some(ValidationSlotType::Rig),
        ItemSlotType::SubSystem => Some(ValidationSlotType::SubSystem),
        ItemSlotType::Service => Some(ValidationSlotType::Service),
        ItemSlotType::TacticalMode => Some(ValidationSlotType::TacticalMode),
    }
}

fn output_validation_slot_type(item: &Item) -> Option<ValidationSlotType> {
    match item.slot.slot_type {
        SlotType::High => Some(ValidationSlotType::High),
        SlotType::Medium => Some(ValidationSlotType::Medium),
        SlotType::Low => Some(ValidationSlotType::Low),
        SlotType::Rig => Some(ValidationSlotType::Rig),
        SlotType::SubSystem => Some(ValidationSlotType::SubSystem),
        SlotType::Service => Some(ValidationSlotType::Service),
        SlotType::TacticalMode => Some(ValidationSlotType::TacticalMode),
        SlotType::DroneBay { .. } => Some(ValidationSlotType::Drone),
        SlotType::Fighter { .. } => Some(ValidationSlotType::Fighter),
        SlotType::Implant => Some(ValidationSlotType::Implant),
        SlotType::Booster => Some(ValidationSlotType::Booster),
        SlotType::Charge | SlotType::Fake => None,
    }
}

fn is_primary_module_slot(item: &Item) -> bool {
    matches!(
        item.slot.slot_type,
        SlotType::High | SlotType::Medium | SlotType::Low
    )
}

fn item_group_id(context: &ValidationContext<'_>, item: &Item) -> Option<i32> {
    if matches!(item.slot.slot_type, SlotType::Fake) {
        return None;
    }
    Some(
        context
            .info
            .get_type(item.item_id.as_type_id(context.fit))
            .group_id,
    )
}

fn item_attribute(item: &Item, attribute_id: i32) -> Option<f64> {
    item.attributes
        .get(&attribute_id)
        .map(|attribute| attribute.value.unwrap_or(attribute.base_value))
}

fn max_active_limit(item: &Item) -> Option<usize> {
    item_attribute(item, ATTR_MAX_ACTIVE)
        .map(|value| value as usize)
        .filter(|&limit| limit > 0)
}

fn item_accepts_charge(item: &Item) -> bool {
    !item_accepted_charge_groups(item).is_empty()
}

fn item_consumes_charge(item: &Item) -> bool {
    item_accepts_charge(item)
        && item_attribute(item, ATTR_CHARGE_RATE).is_some_and(|value| value > 0.0)
}

fn item_accepted_charge_groups(item: &Item) -> Vec<i32> {
    ATTR_CHARGE_GROUPS
        .iter()
        .filter_map(|attribute_id| {
            item_attribute(item, *attribute_id).map(|value| value as i32)
        })
        .filter(|&group_id| group_id != 0)
        .collect()
}

fn booster_slot(type_id: i32, info: &dyn InfoProvider) -> Option<i32> {
    info.get_dogma_attributes(type_id).iter().find_map(|attr| {
        (attr.attribute_id == ATTR_BOOSTER_SLOT).then_some(attr.value as i32)
    })
}

fn find_attr(attributes: &[TypeDogmaAttribute], attribute_id: i32) -> Option<f64> {
    attributes
        .iter()
        .find_map(|attr| (attr.attribute_id == attribute_id).then_some(attr.value))
}
