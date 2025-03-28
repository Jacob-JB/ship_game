use bevy::prelude::*;

pub fn build(app: &mut App) {
    app.insert_resource(BreachRate { rate: 0.1 });
    app.insert_resource(VentFillRate { rate: 0.05 });

    app.add_systems(
        Update,
        (fill_atmospheres, drain_breached_atmospheres).chain(),
    );
}

#[derive(Component, Default)]
pub struct TankAtmosphere {
    /// maximum `level` of the tank
    pub volume: f32,
    /// how many grid atmospheres are in the tank
    pub level: f32,
    /// whether the tank is being used to fill rooms
    pub enabled: bool,
}

#[derive(Component)]
pub struct ModuleAtmosphere {
    pub volume: f32,
    pub level: f32,
    pub breached: bool,
}

/// Vent in a ship module.
///
/// Insert onto ship modules that contain vents.
/// If a ship module doesn't contain a vent it can only be filled
/// from other modules.
#[derive(Component)]
pub struct ModuleVent {
    pub open: bool,
}

#[derive(Resource)]
pub struct VentFillRate {
    pub rate: f32,
}

#[derive(Resource)]
pub struct BreachRate {
    /// Grid spaces of atmosphere lost per second
    pub rate: f32,
}

fn drain_breached_atmospheres(
    mut module_q: Query<&mut ModuleAtmosphere>,
    breach_speed: Res<BreachRate>,
    time: Res<Time>,
) {
    for mut atmosphere in module_q.iter_mut() {
        if atmosphere.breached {
            atmosphere.level = (atmosphere.level - breach_speed.rate * time.delta_secs()).max(0.);
        }
    }
}

fn fill_atmospheres(
    mut module_q: Query<(&ModuleVent, &mut ModuleAtmosphere)>,
    fill_rate: Res<VentFillRate>,
    mut tank_q: Query<&mut TankAtmosphere>,
    time: Res<Time>,
) {
    let compute_module_flow = |atmosphere: &ModuleAtmosphere| {
        let difference = (atmosphere.volume - atmosphere.level).max(0.);
        let max_flow = fill_rate.rate * atmosphere.volume * time.delta_secs();
        difference.min(max_flow)
    };

    let available_atmosphere = tank_q
        .iter()
        .filter_map(|tank| tank.enabled.then_some(tank.level))
        .sum::<f32>();

    let required_atmosphere = module_q
        .iter()
        .filter_map(|(vent, atmosphere)| {
            if !vent.open {
                return None;
            }

            Some(compute_module_flow(atmosphere))
        })
        .sum::<f32>();

    if required_atmosphere == 0.0 {
        // no atmosphere required, return to avoid NaN values.
        return;
    }

    if available_atmosphere == 0.0 {
        // no atmosphere available, return to avoid NaN values.
        return;
    }

    // percent of required atmosphere that can be delivered
    let fill_percent = (available_atmosphere / required_atmosphere).min(1.);
    // percent of atmosphere tanks to be drained
    let drain_percent = (required_atmosphere * fill_percent) / available_atmosphere;

    for mut tank in tank_q.iter_mut() {
        if tank.enabled {
            tank.level -= tank.level * drain_percent;
        }
    }

    for (vent, mut atmosphere) in module_q.iter_mut() {
        if !vent.open {
            continue;
        }

        atmosphere.level += compute_module_flow(atmosphere.as_ref()) * fill_percent;
    }
}
