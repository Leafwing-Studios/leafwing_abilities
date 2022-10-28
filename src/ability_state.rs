// Docs are missing from generated types :(
#![allow(missing_docs)]

use crate::{charges::ChargeState, cooldown::CooldownState, Abilitylike, CannotUseAbility};
use bevy::ecs::query::WorldQuery;
use leafwing_input_manager::action_state::ActionState;

/// A custom [`WorldQuery`](bevy::ecs::query::WorldQuery) type that fetches all ability relevant data for you.
///
/// This type is intended to make collecting the data for [`Abilitylike`] methods easier when working with a full [`AbilitiesBundle`](crate::AbilitiesBundle`).
/// This struct can be used as the first type parameter in a [`Query`](bevy::ecs::system::Query) to fetch the appropriate data.
///
/// Once you have a [`AbilityStateItem`] by calling `.iter_mut()` or `.single_mut` on your query
/// (or a [`AbilityStateReadOnlyItem`] by calling `.iter()` or `.single`),
/// you can use the methods defined there to perform common tasks quickly and reliably.
#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct AbilityState<A: Abilitylike> {
    /// The [`ActionState`] of the abilities of this entity of type `A`
    pub action_state: &'static ActionState<A>,
    /// The [`ChargeState`] associated with each action of type `A` for this entity
    pub charges: &'static mut ChargeState<A>,
    /// The [`CooldownState`] associated with each action of type `A` for this entity
    pub cooldowns: &'static mut CooldownState<A>,
}

impl<A: Abilitylike> AbilityStateItem<'_, A> {
    /// Is this ability ready?
    ///
    /// Calls [`Abilitylike::ready`] on the specified action.
    #[inline]
    pub fn ready(&self, action: A) -> Result<(), CannotUseAbility> {
        action.ready(&*self.charges, &*self.cooldowns)
    }

    /// Is this ability both ready and pressed?
    ///
    /// The error value for "this ability is not ready" will be prioritized over "this ability is not pressed".
    #[inline]
    pub fn ready_and_pressed(&self, action: A) -> Result<(), CannotUseAbility> {
        self.ready(action.clone())?;

        match self.action_state.pressed(action) {
            true => Ok(()),
            false => Err(CannotUseAbility::NotPressed),
        }
    }

    /// Is this ability both ready and just pressed?
    ///
    /// The error value for "this ability is not ready" will be prioritized over "this ability is not pressed".
    #[inline]
    pub fn ready_and_just_pressed(&self, action: A) -> Result<(), CannotUseAbility> {
        self.ready(action.clone())?;

        match self.action_state.just_pressed(action) {
            true => Ok(()),
            false => Err(CannotUseAbility::NotPressed),
        }
    }

    /// Triggers this ability, depleting a charge if available.
    ///
    /// Calls [`Abilitylike::trigger`] on the specified action.
    #[inline]
    pub fn trigger(&mut self, action: A) -> Result<(), CannotUseAbility> {
        action.trigger(&mut *self.charges, &mut *self.cooldowns)
    }

    /// Triggers this ability (and depletes available charges), if action is pressed.
    ///
    /// Calls [`Abilitylike::trigger`] on the specified action.
    #[inline]
    pub fn trigger_if_pressed(&mut self, action: A) -> Result<(), CannotUseAbility> {
        if self.action_state.just_pressed(action.clone()) {
            action.trigger(&mut *self.charges, &mut *self.cooldowns)
        } else {
            Err(CannotUseAbility::NotPressed)
        }
    }

    /// Triggers this ability (and depletes available charges), if action was just pressed.
    ///
    /// Calls [`Abilitylike::trigger`] on the specified action.
    #[inline]
    pub fn trigger_if_just_pressed(&mut self, action: A) -> Result<(), CannotUseAbility> {
        if self.action_state.just_pressed(action.clone()) {
            action.trigger(&mut *self.charges, &mut *self.cooldowns)
        } else {
            Err(CannotUseAbility::NotPressed)
        }
    }
}

impl<A: Abilitylike> AbilityStateReadOnlyItem<'_, A> {
    /// Is this ability ready?
    ///
    /// Calls [`Abilitylike::ready`] on the specified action.
    #[inline]
    pub fn ready(&self, action: A) -> Result<(), CannotUseAbility> {
        action.ready(self.charges, self.cooldowns)
    }

    /// Is this ability both ready and pressed?
    ///
    /// The error value for "this ability is not ready" will be prioritized over "this ability is not pressed".
    #[inline]
    pub fn ready_and_pressed(&self, action: A) -> Result<(), CannotUseAbility> {
        self.ready(action.clone())?;

        match self.action_state.pressed(action) {
            true => Ok(()),
            false => Err(CannotUseAbility::NotPressed),
        }
    }

    /// Is this ability both ready and just pressed?
    ///
    /// The error value for "this ability is not ready" will be prioritized over "this ability is not pressed".
    #[inline]
    pub fn ready_and_just_pressed(&self, action: A) -> Result<(), CannotUseAbility> {
        self.ready(action.clone())?;

        match self.action_state.just_pressed(action) {
            true => Ok(()),
            false => Err(CannotUseAbility::NotPressed),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate as leafwing_abilities;
    use crate::{AbilitiesBundle, AbilityState, Abilitylike};
    use bevy::prelude::*;
    use leafwing_input_manager::{action_state::ActionState, Actionlike};

    #[derive(Actionlike, Abilitylike, Clone, Debug)]
    enum TestAction {
        Duck,
        Cover,
    }

    #[test]
    fn ability_state_methods_are_visible_from_query() {
        fn simple_system(mut query: Query<AbilityState<TestAction>>) {
            let mut ability_state = query.single_mut();
            let _triggered = ability_state.trigger(TestAction::Duck);
        }

        let mut app = App::new();
        app.add_system(simple_system);
    }

    #[test]
    fn ability_state_fetches_abilities_bundle() {
        let mut world = World::new();
        world
            .spawn()
            .insert_bundle(AbilitiesBundle::<TestAction>::default())
            .insert(ActionState::<TestAction>::default());

        let mut query_state = world.query::<AbilityState<TestAction>>();
        assert_eq!(query_state.iter(&world).len(), 1);
    }
}
