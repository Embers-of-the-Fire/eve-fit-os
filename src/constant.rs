pub mod patches {
    pub mod attr {
        include!(concat!(env!("OUT_DIR"), "/patch_attrs.rs"));
    }

    pub mod effect {
        include!(concat!(env!("OUT_DIR"), "/patch_effects.rs"));
    }
}

// ########################
// Type ID Constants
// ########################

/// Internal character representation ID.
pub const CHARACTER_TYPE_ID: i32 = 1373;

// ########################
// Attribute ID Constants
// ########################

/// Mass attribute ID (corresponding to `mass`)
pub const ATTRIBUTE_MASS_ID: i32 = 4;

/// Capacitor need ID
pub const ATTRIBUTE_CAPACITOR_NEED_ID: i32 = 6;

/// Capacity attribute ID (corresponding to `capacity`)
pub const ATTRIBUTE_CAPACITY_ID: i32 = 38;

/// Volume attribute ID (corresponding to `volume`)
pub const ATTRIBUTE_VOLUME_ID: i32 = 161;

/// Radius attribute ID (corresponding to `radius`)
pub const ATTRIBUTE_RADIUS_ID: i32 = 162;

/// Skill level attribute ID (corresponding to `skillLevel`)
pub const ATTRIBUTE_SKILL_LEVEL_ID: i32 = 280;

/// Recharge rate attribute ID (capacitor recharge rate)
pub const ATTRIBUTE_RECHARGE_RATE_ID: i32 = 55;

/// Capacitor capacity attribute name (base capacitor value)
pub const ATTRIBUTE_CAPACITOR_CAPACITY_ID: i32 = 482;

// #######################
// Skill-related Constants
// #######################

/// Attribute ID list for skill dependencies
/// (requiredSkill1, requiredSkill2, etc.)
pub const ATTRIBUTE_SKILLS: [i32; 6] = [182, 183, 184, 1285, 1289, 1290];

// ##########################
// Category Exemption Constants
// ##########################

/// Category ID list exempt from stacking penalties
/// (Ship/Charge/Skill/Implant/Subsystem)
pub const EXEMPT_PENALTY_CATEGORY_IDS: [i32; 5] = [6, 8, 16, 20, 32];

/// Drone category ID (used to identify drone items)
pub const CATEGORY_ID_DRONES: i32 = 18;

// ########################
// Math Constants
// ########################

/// Stacking penalty factor: 1 / e^( (1/2.67)^2 )
pub const PENALTY_FACTOR: f64 = 0.869_119_980_800_397_4;
