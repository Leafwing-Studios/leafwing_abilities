//! The systems that power each [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).

use crate::pool::RegeneratingPool;
use crate::{charges::ChargeState, cooldown::CooldownState, Abilitylike};

use bevy::ecs::prelude::*;
use bevy::time::Time;

/// Advances all [`CooldownState`] components and resources for ability type `A`.
pub fn tick_cooldowns<A: Abilitylike>(
    mut query: Query<
        (Option<&mut CooldownState<A>>, Option<&mut ChargeState<A>>),
        Or<(With<CooldownState<A>>, With<ChargeState<A>>)>,
    >,
    cooldowns_res: Option<ResMut<CooldownState<A>>>,
    charges_res: Option<ResMut<ChargeState<A>>>,
    time: Res<Time>,
) {
    let delta_time = time.delta();

    // Only tick the Cooldowns resource if it exists
    if let Some(mut cooldowns) = cooldowns_res {
        let charges = charges_res.map(|res| res.into_inner());

        cooldowns.tick(delta_time, charges);
    }

    // Only tick the Cooldowns components if they exist
    for (cooldowns, charges) in query.iter_mut() {
        if let Some(mut cooldowns) = cooldowns {
            let charges = charges.map(|data| data.into_inner());

            cooldowns.tick(delta_time, charges);
        }
    }
}

/// Regenerates the resource of the [`Pool`] type `P` based on the elapsed [`Time`].
pub fn regenerate_resource_pool<P: RegeneratingPool + Component + Resource>(
    mut query: Query<&mut P>,
    pool_res: Option<ResMut<P>>,
    time: Res<Time>,
) {
    let delta_time = time.delta();

    for mut pool in query.iter_mut() {
        pool.regenerate(delta_time);
    }

    if let Some(mut pool) = pool_res {
        pool.regenerate(delta_time);
    }
}
