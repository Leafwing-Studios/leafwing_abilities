// Docs are missing from generated types :(
#![allow(missing_docs)]

use crate::{
    charges::ChargeState,
    cooldown::CooldownState,
    pool::{AbilityCosts, MaxPoolLessThanMin, Pool},
    Abilitylike, CannotUseAbility,
};
// Required due to poor macro hygiene in `WorldQuery` macro
// Tracked in https://github.com/bevyengine/bevy/issues/6593
use bevy::{ecs::component::Component, ecs::query::QueryData};
use leafwing_input_manager::action_state::ActionState;

/// A custom [`WorldQuery`](bevy::ecs::query::WorldQuery) type that fetches all ability relevant data for you.
///
/// This type is intended to make collecting the data for [`Abilitylike`] methods easier when working with a full [`AbilitiesBundle`](crate::AbilitiesBundle`).
/// This struct can be used as the first type parameter in a [`Query`](bevy::ecs::system::Query) to fetch the appropriate data.
///
/// If you want your abilities to require paying costs, pass in the appropriate [`Pool`] type `P`.
/// Otherwise, don't specify `P`.
///
/// Once you have a [`AbilityStateItem`] by calling `.iter_mut()` or `.single_mut` on your query
/// (or a [`AbilityStateReadOnlyItem`] by calling `.iter()` or `.single`),
/// you can use the methods defined there to perform common tasks quickly and reliably.
///
/// ## No resource pool
///
/// When working with abilities that don't require a resource pool, simply pass in [`NullPool`] as the pool type.
/// The absence of a pool will be handled gracefully by the methods in [`Abilitylike`].
///
/// ## Multiple resource pools
///
/// When working with abilities that require multiple resource pools, there are two options:
///
/// 1. Create a new [`Pool`] type that contains all of the possible resource pools.
/// 2. Pass in [`NullPool`] and handle the resource costs manually in your ability implementations.
///
/// The first solution is reliable and type-safe, but limits you to a fixed collection of resource pools
/// and can be wasteful and confusing, as the majority of abilities or characters will only use a single resource pool.
///
/// The second solution is more flexible, but requires you to handle the resource costs manually.
/// Make sure to check if the resource cost can be paid before calling [`Abilitylike::trigger`]!
#[derive(QueryData)]
#[query_data(mutable)]
pub struct AbilityState<A: Abilitylike, P: Pool = NullPool> {
    /// The [`ActionState`] of the abilities of this entity of type `A`
    pub action_state: &'static ActionState<A>,
    /// The [`ChargeState`] associated with each action of type `A` for this entity
    pub charges: &'static mut ChargeState<A>,
    /// The [`CooldownState`] associated with each action of type `A` for this entity
    pub cooldowns: &'static mut CooldownState<A>,
    /// The [`Pool`] of resources of type `P` that should be spent
    pub pool: Option<&'static mut P>,
    /// The [`AbilityCosts`] of each ability, in terms of [`P::Quantity`](Pool::Quantity)
    pub ability_costs: Option<&'static mut AbilityCosts<A, P>>,
}

impl<A: Abilitylike, P: Pool> AbilityStateItem<'_, A, P> {
    /// Is this ability ready?
    ///
    /// Calls [`Abilitylike::ready`] on the specified action.
    #[inline]
    pub fn ready(&self, action: &A) -> Result<(), CannotUseAbility> {
        let maybe_pool = self.pool.as_deref();
        let maybe_ability_costs = self.ability_costs.as_deref();

        action.ready(
            &*self.charges,
            &*self.cooldowns,
            maybe_pool,
            maybe_ability_costs,
        )
    }

    /// Is this ability both ready and pressed?
    ///
    /// The error value for "this ability is not pressed" will be prioritized over "this ability is not ready".
    #[inline]
    pub fn ready_and_pressed(&self, action: &A) -> Result<(), CannotUseAbility> {
        if self.action_state.pressed(action) {
            self.ready(action)?;
            Ok(())
        } else {
            Err(CannotUseAbility::NotPressed)
        }
    }

    /// Is this ability both ready and just pressed?
    ///
    /// The error value for "this ability is not pressed" will be prioritized over "this ability is not ready".
    #[inline]
    pub fn ready_and_just_pressed(&self, action: &A) -> Result<(), CannotUseAbility> {
        if self.action_state.just_pressed(action) {
            self.ready(action)?;
            Ok(())
        } else {
            Err(CannotUseAbility::NotPressed)
        }
    }

    /// Triggers this ability, depleting a charge if available.
    ///
    /// Calls [`Abilitylike::trigger`] on the specified action.
    #[inline]
    pub fn trigger(&mut self, action: &A) -> Result<(), CannotUseAbility> {
        let maybe_pool = self.pool.as_deref_mut();
        let maybe_ability_costs = self.ability_costs.as_deref();

        action.trigger(
            &mut *self.charges,
            &mut *self.cooldowns,
            maybe_pool,
            maybe_ability_costs,
        )
    }

    /// Triggers this ability (and depletes available charges), if action is pressed.
    ///
    /// Calls [`Abilitylike::trigger`] on the specified action.
    #[inline]
    pub fn trigger_if_pressed(&mut self, action: &A) -> Result<(), CannotUseAbility> {
        if self.action_state.just_pressed(action) {
            let maybe_pool = self.pool.as_deref_mut();
            let maybe_ability_costs = self.ability_costs.as_deref();

            action.trigger(
                &mut *self.charges,
                &mut *self.cooldowns,
                maybe_pool,
                maybe_ability_costs,
            )
        } else {
            Err(CannotUseAbility::NotPressed)
        }
    }

    /// Triggers this ability (and depletes available charges), if action was just pressed.
    ///
    /// Calls [`Abilitylike::trigger`] on the specified action.
    #[inline]
    pub fn trigger_if_just_pressed(&mut self, action: &A) -> Result<(), CannotUseAbility> {
        if self.action_state.just_pressed(action) {
            let maybe_pool = self.pool.as_deref_mut();
            let maybe_ability_costs = self.ability_costs.as_deref();

            action.trigger(
                &mut *self.charges,
                &mut *self.cooldowns,
                maybe_pool,
                maybe_ability_costs,
            )
        } else {
            Err(CannotUseAbility::NotPressed)
        }
    }
}

impl<A: Abilitylike, P: Pool> AbilityStateReadOnlyItem<'_, A, P> {
    /// Is this ability ready?
    ///
    /// Calls [`Abilitylike::ready`] on the specified action.
    #[inline]
    pub fn ready(&self, action: &A) -> Result<(), CannotUseAbility> {
        action.ready(self.charges, self.cooldowns, self.pool, self.ability_costs)
    }

    /// Is this ability both ready and pressed?
    ///
    /// The error value for "this ability is not pressed" will be prioritized over "this ability is not ready".
    #[inline]
    pub fn ready_and_pressed(&self, action: &A) -> Result<(), CannotUseAbility> {
        if self.action_state.pressed(action) {
            self.ready(action)?;
            Ok(())
        } else {
            Err(CannotUseAbility::NotPressed)
        }
    }

    /// Is this ability both ready and just pressed?
    ///
    /// The error value for "this ability is not pressed" will be prioritized over "this ability is not ready".
    #[inline]
    pub fn ready_and_just_pressed(&self, action: &A) -> Result<(), CannotUseAbility> {
        if self.action_state.just_pressed(action) {
            self.ready(action)?;
            Ok(())
        } else {
            Err(CannotUseAbility::NotPressed)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate as leafwing_abilities;
    use crate::{AbilitiesBundle, AbilityState, Abilitylike};
    use bevy::{prelude::*, reflect::Reflect};
    use leafwing_input_manager::{action_state::ActionState, Actionlike};

    #[derive(Actionlike, Reflect, Abilitylike, Clone, Debug, Hash, PartialEq, Eq)]
    enum TestAction {
        Duck,
        Cover,
    }

    #[test]
    fn ability_state_methods_are_visible_from_query() {
        fn simple_system(mut query: Query<AbilityState<TestAction>>) {
            let mut ability_state = query.single_mut().unwrap();
            let _triggered = ability_state.trigger(&TestAction::Duck);
        }

        let mut app = App::new();
        app.add_systems(Update, simple_system);
    }

    #[test]
    fn ability_state_fetches_abilities_bundle() {
        let mut world = World::new();
        world
            .spawn(AbilitiesBundle::<TestAction>::default())
            .insert(ActionState::<TestAction>::default());

        let mut query_state = world.query::<AbilityState<TestAction>>();
        assert_eq!(query_state.iter(&world).len(), 1);
    }
}

/// A no-op type that implements [`Pool`] and [`Component`].
///
/// Used in [`AbilityState`] to get the type system to play nice when no resource pool type is needed.
///
/// Values of this type should never be constructed.
#[derive(Component, Debug, Default)]
pub struct NullPool;

impl Pool for NullPool {
    type Quantity = f32;
    const MIN: f32 = 0.0;

    fn current(&self) -> Self::Quantity {
        Self::MIN
    }

    fn set_current(&mut self, _new_quantity: Self::Quantity) -> Self::Quantity {
        Self::MIN
    }

    fn max(&self) -> Self::Quantity {
        Self::MIN
    }

    fn set_max(&mut self, _new_max: Self::Quantity) -> Result<(), MaxPoolLessThanMin> {
        Ok(())
    }
}
