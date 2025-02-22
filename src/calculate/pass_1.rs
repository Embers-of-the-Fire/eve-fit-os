use super::Ship;
use super::item::{Attribute, EffectCategory, Item, Slot};
use crate::constant::patches::attr::{
    ATTR_DAMAGE_PROFILE_EM, ATTR_DAMAGE_PROFILE_EXPLOSIVE, ATTR_DAMAGE_PROFILE_KINETIC,
    ATTR_DAMAGE_PROFILE_THERMAL,
};
use crate::constant::{
    ATTRIBUTE_CAPACITY_ID, ATTRIBUTE_MASS_ID, ATTRIBUTE_RADIUS_ID,
    ATTRIBUTE_SKILL_LEVEL_ID, ATTRIBUTE_VOLUME_ID,
};
use crate::provider::{FitProvider, InfoProvider};

impl Item {
    pub(super) fn set_attribute(&mut self, attribute_id: i32, value: f64) {
        self.attributes
            .insert(attribute_id, Attribute::new_base(value));
    }

    pub(super) fn update_attributes(&mut self, info: &impl InfoProvider) {
        for dogma_attribute in info.get_dogma_attributes(self.type_id) {
            self.set_attribute(dogma_attribute.attribute_id, dogma_attribute.value);
        }

        // Some attributes of items come from the Type information.
        let ty = info.get_type(self.type_id);
        if let Some(mass) = ty.mass {
            self.set_attribute(ATTRIBUTE_MASS_ID, mass);
        }
        if let Some(capacity) = ty.capacity {
            self.set_attribute(ATTRIBUTE_CAPACITY_ID, capacity);
        }
        if let Some(volume) = ty.volume {
            self.set_attribute(ATTRIBUTE_VOLUME_ID, volume);
        }
        if let Some(radius) = ty.radius {
            self.set_attribute(ATTRIBUTE_RADIUS_ID, radius);
        }
    }
}

pub(crate) fn pass(fit: &impl FitProvider, info: &impl InfoProvider, ship: &mut Ship) {
    ship.hull.update_attributes(info);
    
    [
        (ATTR_DAMAGE_PROFILE_EM, ship.damage_profile.em),
        (ATTR_DAMAGE_PROFILE_EXPLOSIVE, ship.damage_profile.explosive),
        (ATTR_DAMAGE_PROFILE_KINETIC, ship.damage_profile.kinetic),
        (ATTR_DAMAGE_PROFILE_THERMAL, ship.damage_profile.thermal),
    ]
    .into_iter()
    .for_each(|(id, val)| ship.hull.set_attribute(id, val));

    for (skill_id, skill_level) in fit.skills() {
        let mut skill = Item::new_fake(*skill_id);
        skill.update_attributes(info);
        skill.set_attribute(ATTRIBUTE_SKILL_LEVEL_ID, *skill_level as f64);
        ship.skills.push(skill);
    }

    for module in &fit.fit().modules {
        let state = module.state.into();

        let mut item = Item::new_module(
            module.type_id,
            Slot {
                slot_type: module.slot.slot_type.into(),
                index: Some(module.slot.index),
            },
            module.charge.map(|c| c.type_id),
            state,
        );

        item.update_attributes(info);
        if let Some(charge) = &mut item.charge {
            charge.update_attributes(info);
        }

        ship.modules.push(item);
    }

    for drone in &fit.fit().drones {
        let state: EffectCategory = drone.state.into();
        let state = state.fallback_active();

        let mut item = Item::new_drone(drone.type_id, drone.group_id, state);
        item.update_attributes(info);

        ship.modules.push(item);
    }

    for implant in &fit.fit().implants {
        let mut item = Item::new_implant(implant.type_id, implant.index);
        item.update_attributes(info);
        ship.implants.push(item);
    }
}
