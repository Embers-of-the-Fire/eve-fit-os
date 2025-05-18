use super::Ship;
use crate::provider::{FitProvider, InfoProvider};

mod capacitor;

/// Attributes don't contain all information displayed, so we calculate some fake attributes with those values.
pub(super) fn pass(_fit: &impl FitProvider, _info: &impl InfoProvider, ship: &mut Ship) {
    capacitor::attribute_capacitor_depletes_in(ship);
}
