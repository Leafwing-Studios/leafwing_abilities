//! The systems that power each [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).

use crate::pool::RegeneratingPool;
use crate::{charges::ChargeState, cooldown::CooldownState, Abilitylike};

use bevy::ecs::prelude::*;
use bevy::time::Time;

/// Advances all [`CooldownState`] components for ability type `A`.
pub fn tick_cooldowns<A: Abilitylike>(
    mut query: Query<
        (Option<&mut CooldownState<A>>, Option<&mut ChargeState<A>>),
        Or<(With<CooldownState<A>>, With<ChargeState<A>>)>,
    >,
    time: Res<Time>,
) {
    let delta_time = time.delta();

    // Only tick the Cooldowns components if they exist
    for (cooldowns, charges) in query.iter_mut() {
        if let Some(mut cooldowns) = cooldowns {
            let charges = charges.map(|data| data.into_inner());

            cooldowns.tick(delta_time, charges);
        }
    }
}

/// Regenerates the [`RegeneratingPool`] type `P` based on the elapsed [`Time`].
pub fn regenerate_resource_pool<P: RegeneratingPool>(mut query: Query<&mut P>, time: Res<Time>) {
    let delta_time = time.delta();

    for mut pool in query.iter_mut() {
        pool.regenerate(delta_time);
    }
}
