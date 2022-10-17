//! The systems that power each [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).

use crate::{charges::ChargeState, cooldown::CooldownState, Abilitylike};

use bevy::ecs::{prelude::*, schedule::ShouldRun};
use bevy::time::Time;
use leafwing_input_manager::plugin::ToggleActions;

/// Advances all [`Cooldowns`].
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

/// Returns [`ShouldRun::No`] if [`DisableInput`] exists and [`ShouldRun::Yes`] otherwise
pub(super) fn run_if_enabled<A: Abilitylike>(toggle_actions: Res<ToggleActions<A>>) -> ShouldRun {
    if toggle_actions.enabled {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}
