use bevy::prelude::*;

pub fn build(app: &mut App) {
    app.insert_resource(BreachRate { rate: 0.1 });
    app.insert_resource(VentFillRate { rate: 0.05 });

    app.add_systems(
        Update,
        (fill_atmospheres, drain_breached_atmospheres).chain(),
    );
}

#[derive(Component)]
pub struct ModuleAtmosphere {
    pub volume: f32,
    pub level: f32,
    pub breached: bool,
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

/// Vent in a ship module.
#[derive(Component)]
pub struct Vent {
    pub open: bool,
}

#[derive(Resource)]
pub struct VentFillRate {
    pub rate: f32,
}

#[derive(Component)]
pub struct AtmosphereTank {
    pub volume: f32,
    pub level: f32,
    pub enabled: bool,
}

fn fill_atmospheres(
    mut module_q: Query<(&Vent, &mut ModuleAtmosphere)>,
    fill_rate: Res<VentFillRate>,
    mut tank_q: Query<&mut AtmosphereTank>,
    time: Res<Time>,
) {
    let vent_flow = |atmosphere: &ModuleAtmosphere| {
        let difference = (atmosphere.level - atmosphere.volume).max(0.);
        difference.min(difference * fill_rate.rate * time.delta_secs())
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

            Some(vent_flow(atmosphere))
        })
        .sum::<f32>();

    // percent of required atmosphere that can be delivered
    let fill_percent = (available_atmosphere / required_atmosphere).min(1.);
    // percent of atmosphere tanks to be drained
    let drain_percent = (required_atmosphere * fill_percent) / available_atmosphere;

    for mut tank in tank_q.iter_mut() {
        tank.level -= tank.level * drain_percent;
    }

    for (vent, mut atmosphere) in module_q.iter_mut() {
        if !vent.open {
            continue;
        }

        atmosphere.level += vent_flow(atmosphere.as_ref()) * fill_percent;
    }
}
