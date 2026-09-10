use super::Ship;
use crate::provider::{FitProvider, InfoProvider};

mod capacitor;
mod precursor_turret;

/// Attributes don't contain all information displayed, so we calculate some fake attributes with those values.
pub(super) fn pass(fit: &impl FitProvider, _info: &impl InfoProvider, ship: &mut Ship) {
    capacitor::attribute_capacitor_depletes_in(ship);
    precursor_turret::attribute_precursor_turret_spool(fit, ship);
}
