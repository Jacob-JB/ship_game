use std::{collections::VecDeque, time::Duration};

use bevy::prelude::*;
use common::physics::*;

use crate::networking::prelude::*;

pub fn build(app: &mut App) {
    app.init_resource::<PhysicsTimeEstimate>();
    app.init_resource::<PhysicsSnapshots>();

    app.add_systems(
        Update,
        (
            receive_time_samples,
            calculate_time_estimate,
            receive_physics_snapshots,
        ),
    );
}

/// how many samples to keep to make the time estimate
const TIME_ESTIMATE_SAMPLES: usize = 30;

#[derive(Resource, Default)]
pub struct PhysicsTimeEstimate {
    /// list of samples and when they were received
    samples: VecDeque<(TimeSample, Duration)>,
    pub time_estimate: Duration,
}

fn receive_time_samples(
    mut messages: MessageReceiver<TimeSample>,
    mut physics_time: ResMut<PhysicsTimeEstimate>,
    time: Res<Time>,
) {
    for sample in messages.drain() {
        physics_time.samples.push_back((sample, time.elapsed()));

        if physics_time.samples.len() > TIME_ESTIMATE_SAMPLES {
            physics_time.samples.pop_front();
        }
    }
}

fn calculate_time_estimate(mut physics_time: ResMut<PhysicsTimeEstimate>, time: Res<Time>) {
    if let Some(time_estimate) = physics_time
        .samples
        .iter()
        .map(|(sample, received_time)| sample.time + (time.elapsed() - *received_time))
        .sum::<Duration>()
        .checked_div(physics_time.samples.len() as u32)
    {
        physics_time.time_estimate = time_estimate;
    }
}

#[derive(Resource, Default)]
pub struct PhysicsSnapshots {
    /// ordered list of physics snapshots sent from the server
    pub snapshots: VecDeque<PhysicsSnapshot>,
}

impl PhysicsSnapshots {
    /// binary searches for the index of a specific time
    ///
    /// if there is an exact match returns that index, otherwise the index directly after
    pub fn search(&self, time: Duration) -> usize {
        let (Ok(index) | Err(index)) = self
            .snapshots
            .binary_search_by(|snapshot| snapshot.time.cmp(&time));

        index
    }
}

fn receive_physics_snapshots(
    mut messages: MessageReceiver<PhysicsSnapshot>,
    mut snapshots: ResMut<PhysicsSnapshots>,
) {
    for snapshot in messages.drain() {
        let index = snapshots.search(snapshot.time);

        snapshots.snapshots.insert(index, snapshot);
    }
}
