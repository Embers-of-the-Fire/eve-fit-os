use super::item::Attribute;
use super::{Item, Ship};
use crate::provider::{FitProvider, InfoProvider};

mod capacitor;

impl Item {
    pub fn add_attribute(&mut self, attribute_id: i32, base_value: f64, value: f64) {
        let mut attribute = Attribute::new_base(base_value);
        attribute.value = Some(value);
        self.attributes.insert(attribute_id, attribute);
    }
}

/// Attributes don't contain all information displayed, so we calculate some fake attributes with those values.
pub(super) fn pass(
    _fit: &impl FitProvider,
    _info: &impl InfoProvider,
    ship: &mut Ship,
) {
    capacitor::attribute_capacitor_depletes_in(ship);
}
