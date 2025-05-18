use crate::{calculate::Ship, provider::FitProvider};

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
            module.set_attribute(
                ATTR_ARMOR_AMO,
                amo.base_value * mult.base_value,
            );
        }
    }
}
