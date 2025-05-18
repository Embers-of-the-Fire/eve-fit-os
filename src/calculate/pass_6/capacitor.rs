use super::super::Ship;
use crate::constant::patches::attr::{
    ATTR_CAPACITOR_BOOST, ATTR_CAPACITOR_DEPLETES_IN, ATTR_CAPACITOR_PEAK_DELTA,
    ATTR_CYCLE_TIME,
};
use crate::constant::{
    ATTRIBUTE_CAPACITOR_CAPACITY_ID, ATTRIBUTE_CAPACITOR_NEED_ID,
    ATTRIBUTE_RECHARGE_RATE_ID,
};

struct Module {
    capacitor_need: f64,
    duration: f64,
    time_next: f64,
}

/// Amount of seconds it takes for the capacitor to deplete; or negative if it is stable.
pub fn attribute_capacitor_depletes_in(ship: &mut Ship) {
    if !ship
        .hull
        .attributes
        .contains_key(&ATTR_CAPACITOR_PEAK_DELTA)
    {
        return;
    }

    let mut depletes_in = -1000.0;

    let attr_capacitor_peak_delta = ship
        .hull
        .attributes
        .get(&ATTR_CAPACITOR_PEAK_DELTA)
        .unwrap();

    if attr_capacitor_peak_delta.value.unwrap() < 0.0 {
        let attr_capacitor_capacity = ship
            .hull
            .attributes
            .get(&ATTRIBUTE_CAPACITOR_CAPACITY_ID)
            .unwrap();
        let attr_recharge_rate = ship
            .hull
            .attributes
            .get(&ATTRIBUTE_RECHARGE_RATE_ID)
            .unwrap();

        // Find all modules consuming capacitor.
        let mut modules = Vec::new();
        for item in &ship.modules {
            if !item.slot.is_module() || !item.state.is_active() {
                continue;
            }

            if !(item.attributes.contains_key(&ATTRIBUTE_CAPACITOR_NEED_ID)
                || item.attributes.contains_key(&ATTR_CAPACITOR_BOOST))
                || !item.attributes.contains_key(&ATTR_CYCLE_TIME)
            {
                continue;
            }

            let duration = item
                .attributes
                .get(&ATTR_CYCLE_TIME)
                .unwrap()
                .value
                .unwrap();

            let capacitor_need = item
                .attributes
                .get(&ATTRIBUTE_CAPACITOR_NEED_ID)
                .or_else(|| item.attributes.get(&ATTR_CAPACITOR_BOOST))
                .unwrap()
                .value
                .unwrap();

            modules.push(Module {
                capacitor_need,
                duration,
                time_next: 0.0,
            });
        }

        if !modules.is_empty() {
            let capacitor_capacity = attr_capacitor_capacity.value.unwrap();
            let recharge_rate = attr_recharge_rate.value.unwrap();

            let mut capacitor = capacitor_capacity;
            let mut time_last = 0.0;
            let mut time_next = 0.0;

            // Simulate the capacitor to find out when it depletes.
            while capacitor > 0.0 {
                capacitor = (1.0
                    + (f64::sqrt(capacitor / capacitor_capacity) - 1.0)
                        * f64::exp(5.0 * (time_last - time_next) / recharge_rate))
                .powi(2)
                    * capacitor_capacity;

                time_last = time_next;
                time_next = f64::INFINITY;

                for module in &mut modules {
                    if module.time_next <= time_last {
                        module.time_next += module.duration;
                        capacitor -= module.capacitor_need;
                    }

                    // Find the next module that would use capacitor.
                    time_next = f64::min(time_next, module.time_next);
                }

                if time_last > 86400.0 {
                    break;
                }
            }

            depletes_in = time_last;
        }
    }

    ship.hull
        .add_attribute(ATTR_CAPACITOR_DEPLETES_IN, 0.0, depletes_in / 1000.0);
}
