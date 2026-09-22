use crate::calculate::Ship;
use crate::provider::FitProvider;

const TYPE_ID_ARMOR_REPAIRER_CHARGE: i32 = 28668;
const ATTR_ARMOR_MULT: i32 = 1886;
const ATTR_ARMOR_AMO: i32 = 84;

pub fn attribute_chargeable_armor_repairer(fit: &impl FitProvider, ship: &mut Ship) {
    for module in ship.modules.iter_mut() {
        if module
            .charge
            .as_ref()
            .is_some_and(|c| c.item_id.as_type_id(fit) == TYPE_ID_ARMOR_REPAIRER_CHARGE)
        {
            let Some(mult) = module.attributes.get(&ATTR_ARMOR_MULT) else {
                continue;
            };
            let Some(amo) = module.attributes.get(&ATTR_ARMOR_AMO) else {
                continue;
            };
            let amount = amo.base_value * mult.base_value;
            // Multiply the base value in place: replacing the attribute would
            // drop the effects pass 2 already collected on it (rigs, ship
            // bonuses, overload, boosters).
            // Note: the multiplier is read from its base value, so effects
            // modifying ATTR_ARMOR_MULT itself would not be honored here.
            module
                .attributes
                .get_mut(&ATTR_ARMOR_AMO)
                .unwrap()
                .base_value = amount;
        }
    }
}
