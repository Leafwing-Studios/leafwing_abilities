#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![warn(clippy::doc_markdown)]
#![doc = include_str!("../README.md")]

use crate::cooldown::CooldownState;
use bevy::{ecs::prelude::*, reflect::Reflect};
use charges::{ChargeState, Charges};
use cooldown::Cooldown;
use leafwing_input_manager::Actionlike;
use pool::{AbilityCosts, Pool};
use thiserror::Error;

mod ability_state;
pub mod charges;
pub mod cooldown;
pub mod plugin;
pub mod pool;
#[cfg(feature = "premade_pools")]
pub mod premade_pools;
pub mod systems;
pub use ability_state::*;

// Importing the derive macro
pub use leafwing_abilities_macros::Abilitylike;

/// Everything you need to get started
pub mod prelude {
    pub use crate::charges::{ChargeState, Charges};
    pub use crate::cooldown::{Cooldown, CooldownState};
    pub use crate::pool::{AbilityCosts, Pool, PoolBundle};

    pub use crate::plugin::AbilityPlugin;
    pub use crate::CannotUseAbility;
    pub use crate::{AbilitiesBundle, AbilityState, Abilitylike};
}

/// Allows a type to be used as a gameplay action in an input-agnostic fashion
///
/// Actions are modelled as "virtual buttons", cleanly abstracting over messy, customizable inputs
/// in a way that can be easily consumed by your game logic.
///
/// This trait should be implemented on the `A` type that you want to pass into [`InputManagerPlugin`](crate::plugin::InputManagerPlugin).
///
/// Generally, these types will be very small (often data-less) enums.
/// As a result, the APIs in this crate accept actions by value, rather than reference.
/// While `Copy` is not a required trait bound,
/// users are strongly encouraged to derive `Copy` on these enums whenever possible to improve ergonomics.
///
/// # Example
/// ```rust
/// use leafwing_input_manager::Actionlike;
/// use bevy::reflect::Reflect;
///
/// #[derive(Actionlike, Debug, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
/// enum PlayerAction {
///    // Movement
///    Up,
///    Down,
///    Left,
///    Right,
///    // Abilities
///    Ability1,
///    Ability2,
///    Ability3,
///    Ability4,
///    Ultimate,
/// }
/// ```
pub trait Abilitylike: Actionlike {
    /// Is this ability ready?
    ///
    /// If this ability has charges, at least one charge must be available.
    /// If this ability has a cooldown but no charges, the cooldown must be ready.
    /// Otherwise, returns [`Ok(())`].
    ///
    /// Calls [`ability_ready`], which can be used manually if you already know the [`Charges`] and [`Cooldown`] of interest.
    fn ready<P: Pool>(
        &self,
        charges: &ChargeState<Self>,
        cooldowns: &CooldownState<Self>,
        maybe_pool: Option<&P>,
        maybe_costs: Option<&AbilityCosts<Self, P>>,
    ) -> Result<(), CannotUseAbility> {
        let charges = charges.get(self);
        let cooldown = cooldowns.get(self);

        ability_ready(
            charges,
            cooldown,
            maybe_pool,
            maybe_costs.and_then(|costs| costs.get(self)).copied(),
        )
    }

    /// Triggers this ability, depleting a charge if available.
    ///
    /// Returns `true` if the ability could be used, and `false` if it could not be.
    /// Abilities can only be used if they are ready.
    ///     
    /// Calls [`trigger_ability`], which can be used manually if you already know the [`Charges`] and [`Cooldown`] of interest.
    fn trigger<P: Pool>(
        &self,
        charges: &mut ChargeState<Self>,
        cooldowns: &mut CooldownState<Self>,
        maybe_pool: Option<&mut P>,
        maybe_costs: Option<&AbilityCosts<Self, P>>,
    ) -> Result<(), CannotUseAbility> {
        let charges = charges.get_mut(self);
        let cooldown = cooldowns.get_mut(self);

        trigger_ability(
            charges,
            cooldown,
            maybe_pool,
            maybe_costs.and_then(|costs| costs.get(self)).copied(),
        )
    }

    /// Triggers this ability, depleting a charge if available.
    ///
    /// Returns `true` if the ability could be used, and `false` if it could not be.
    /// Abilities can only be used if they are ready.
    ///     
    /// Calls [`Abilitylike::trigger`], passing in [`None`] for both the pools or costs.
    /// This is useful when you don't have any pools or costs to check,
    /// or when multiple distinct pools may be needed.
    fn trigger_no_costs(
        &self,
        charges: &mut ChargeState<Self>,
        cooldowns: &mut CooldownState<Self>,
    ) -> Result<(), CannotUseAbility> {
        self.trigger::<NullPool>(charges, cooldowns, None, None)
    }

    /// Is this ability ready?
    ///
    /// If this ability has charges, at least one charge must be available.
    /// If this ability has a cooldown but no charges, the cooldown must be ready.
    /// Otherwise, returns [`Ok(())`].
    ///
    /// Calls [`Abilitylike::ready`], passing in [`None`] for both the pools or costs.
    /// This is useful when you don't have any pools or costs to check,
    /// or when multiple distinct pools may be needed.
    fn ready_no_costs(
        &self,
        charges: &ChargeState<Self>,
        cooldowns: &CooldownState<Self>,
    ) -> Result<(), CannotUseAbility> {
        self.ready::<NullPool>(charges, cooldowns, None, None)
    }
}

/// An [`Error`](std::error::Error) type that explains why an ability could not be used.
///
/// The priority of these errors follows the order of this enum.
/// For example, if an ability is out of charges and also not pressed,
/// [`ready_and_pressed`](crate::ability_state::AbilityStateItem) will return `Err(CannotUseAbility::NotPressed)`,
/// rather than `Err(CannotUseAbility::NoCharges)`, even though both are true.
#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CannotUseAbility {
    /// The corresponding [`ActionState`](leafwing_input_manager::action_state::ActionState) was not pressed
    #[error("The ability was not pressed.")]
    NotPressed,
    /// There were no [`Charges`] available for this ability
    #[error("No charges available.")]
    NoCharges,
    /// The [`Cooldown`] of this ability was not ready
    #[error("Cooldown not ready.")]
    OnCooldown,
    /// The Global [`Cooldown`] for this [`CooldownState`] was not ready
    #[error("Global cooldown not ready.")]
    OnGlobalCooldown,
    /// Not enough resources from the corresponding [`Pool`]s are available
    #[error("Not enough resources.")]
    PoolInsufficient,
}

/// Checks if a [`Charges`], [`Cooldown`] pair associated with an ability is ready to use.
///
/// If this ability has charges, at least one charge must be available.
/// If this ability has a cooldown but no charges, the cooldown must be ready.
/// Otherwise, returns `true`.
///
/// If you don't have an associated resource pool to check, pass in [`NullPool`] as `P`.
#[inline]
pub fn ability_ready<P: Pool>(
    charges: Option<&Charges>,
    cooldown: Option<&Cooldown>,
    pool: Option<&P>,
    cost: Option<P::Quantity>,
) -> Result<(), CannotUseAbility> {
    if let Some(charges) = charges {
        if charges.charges() > 0 {
            Ok(())
        } else {
            Err(CannotUseAbility::NoCharges)
        }
    } else if let Some(cooldown) = cooldown {
        cooldown.ready()
    } else if let Some(pool) = pool {
        if let Some(cost) = cost {
            pool.available(cost)
        } else {
            Ok(())
        }
    // The pool does not exist, but the cost does
    } else if let Some(cost) = cost {
        if cost > P::MIN {
            Err(CannotUseAbility::PoolInsufficient)
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

/// Triggers an implicit ability, depleting a charge if available.
///
/// If no `charges` is [`None`], this will be based off the [`Cooldown`] alone, triggering it if possible.
/// If you don't have an associated resource pool to check, pass in [`NullPool`] as `P`.
#[inline]
pub fn trigger_ability<P: Pool>(
    mut charges: Option<&mut Charges>,
    mut cooldown: Option<&mut Cooldown>,
    pool: Option<&mut P>,
    cost: Option<P::Quantity>,
) -> Result<(), CannotUseAbility> {
    ability_ready(
        charges.as_deref(),
        cooldown.as_deref(),
        pool.as_deref(),
        cost,
    )?;

    if let Some(ref mut charges) = charges {
        charges.expend()?;
    } else if let Some(ref mut cooldown) = cooldown {
        cooldown.trigger()?;
    }

    if let Some(pool) = pool {
        if let Some(cost) = cost {
            let _pool_result = pool.expend(cost);
            // This is good to check, but panics in release mode are miserable
            debug_assert!(_pool_result.is_ok());
        }
    }

    Ok(())
}

/// This [`Bundle`] allows entities to manage their [`Abilitylike`] actions effectively.
///
/// Commonly combined with an [`InputManagerBundle`](leafwing_input_manager::InputManagerBundle),
/// which tracks whether or not actions are pressed.
///
/// If you would like to track resource costs for your abilities, combine this with a [`PoolBundle`](crate::pool::PoolBundle).
///
/// Use with [`AbilityPlugin`](crate::plugin::AbilityPlugin), providing the same enum type to both.
#[derive(Bundle, Clone, Debug, PartialEq, Eq, Reflect)]
pub struct AbilitiesBundle<A: Abilitylike> {
    /// A [`CooldownState`] component
    pub cooldowns: CooldownState<A>,
    /// A [`ChargeState`] component
    pub charges: ChargeState<A>,
}

// Cannot use derive(Default), as it forces an undesirable bound on our generics
impl<A: Abilitylike> Default for AbilitiesBundle<A> {
    fn default() -> Self {
        Self {
            cooldowns: CooldownState::default(),
            charges: ChargeState::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::reflect::Reflect;
    use leafwing_abilities_macros::Abilitylike;
    use leafwing_input_manager::Actionlike;

    use crate::charges::Charges;
    use crate::cooldown::Cooldown;
    use crate::NullPool;
    use crate::{ability_ready, trigger_ability, CannotUseAbility};

    use crate as leafwing_abilities;

    #[derive(Abilitylike, Actionlike, Reflect, Clone, Hash, PartialEq, Eq, Debug)]
    enum TestAbility {
        TestAction,
    }

    #[test]
    fn abilitylike_works() {}

    #[test]
    fn ability_ready_no_cooldown_no_charges() {
        assert!(ability_ready::<NullPool>(None, None, None, None).is_ok());
    }

    #[test]
    fn ability_ready_just_cooldown() {
        let mut cooldown = Some(Cooldown::from_secs(1.));
        assert!(ability_ready::<NullPool>(None, cooldown.as_ref(), None, None).is_ok());

        cooldown.as_mut().map(|c| c.trigger());
        assert_eq!(
            ability_ready::<NullPool>(None, cooldown.as_ref(), None, None),
            Err(CannotUseAbility::OnCooldown)
        );
    }

    #[test]
    fn ability_ready_just_charges() {
        let mut charges = Some(Charges::simple(1));

        assert!(ability_ready::<NullPool>(charges.as_ref(), None, None, None).is_ok());

        charges.as_mut().map(|c| c.expend());
        assert_eq!(
            ability_ready::<NullPool>(charges.as_ref(), None, None, None),
            Err(crate::CannotUseAbility::NoCharges)
        );
    }

    #[test]
    fn ability_ready_cooldown_and_charges() {
        let mut charges = Some(Charges::simple(1));
        let mut cooldown = Some(Cooldown::from_secs(1.));
        // Both available
        assert!(ability_ready::<NullPool>(charges.as_ref(), cooldown.as_ref(), None, None).is_ok());

        // Out of charges, cooldown ready
        charges.as_mut().map(|c| c.expend());
        assert_eq!(
            ability_ready::<NullPool>(charges.as_ref(), cooldown.as_ref(), None, None),
            Err(CannotUseAbility::NoCharges)
        );

        // Just charges
        if let Some(c) = charges.as_mut() {
            c.replenish()
        }
        cooldown.as_mut().map(|c| c.trigger());
        assert!(ability_ready::<NullPool>(charges.as_ref(), cooldown.as_ref(), None, None).is_ok());

        // Neither
        charges.as_mut().map(|c| c.expend());
        assert_eq!(
            ability_ready::<NullPool>(charges.as_ref(), cooldown.as_ref(), None, None),
            Err(CannotUseAbility::NoCharges)
        );
    }

    #[test]
    fn trigger_ability_no_cooldown_no_charges() {
        let outcome = trigger_ability::<NullPool>(None, None, None, None);
        assert!(outcome.is_ok());
    }

    #[test]
    fn trigger_ability_just_cooldown() {
        let mut cooldown = Some(Cooldown::from_secs(1.));
        assert!(trigger_ability::<NullPool>(None, cooldown.as_mut(), None, None).is_ok());

        cooldown.as_mut().map(|c| c.trigger());
        assert_eq!(
            trigger_ability::<NullPool>(None, cooldown.as_mut(), None, None),
            Err(CannotUseAbility::OnCooldown)
        );
        assert_eq!(
            ability_ready::<NullPool>(None, cooldown.as_ref(), None, None),
            Err(CannotUseAbility::OnCooldown)
        );
    }

    #[test]
    fn trigger_ability_just_charges() {
        let mut charges = Some(Charges::simple(1));

        assert!(trigger_ability::<NullPool>(charges.as_mut(), None, None, None).is_ok());

        charges.as_mut().map(|c| c.expend());
        assert_eq!(
            trigger_ability::<NullPool>(charges.as_mut(), None, None, None),
            Err(CannotUseAbility::NoCharges)
        );
        assert_eq!(
            ability_ready::<NullPool>(charges.as_ref(), None, None, None),
            Err(CannotUseAbility::NoCharges)
        );
    }

    #[test]
    fn trigger_ability_cooldown_and_charges() {
        let mut charges = Some(Charges::simple(1));
        let mut cooldown = Some(Cooldown::from_secs(1.));
        // Both available
        assert!(
            trigger_ability::<NullPool>(charges.as_mut(), cooldown.as_mut(), None, None).is_ok()
        );
        assert_eq!(
            ability_ready::<NullPool>(charges.as_ref(), cooldown.as_ref(), None, None),
            Err(CannotUseAbility::NoCharges)
        );

        // None available
        assert_eq!(
            trigger_ability::<NullPool>(charges.as_mut(), cooldown.as_mut(), None, None),
            Err(CannotUseAbility::NoCharges)
        );

        // Just charges
        if let Some(c) = charges.as_mut() {
            c.replenish()
        }
        assert!(
            trigger_ability::<NullPool>(charges.as_mut(), cooldown.as_mut(), None, None).is_ok()
        );

        // Just cooldown
        charges.as_mut().map(|c| c.expend());
        if let Some(c) = cooldown.as_mut() {
            c.refresh()
        }
        assert_eq!(
            trigger_ability::<NullPool>(charges.as_mut(), cooldown.as_mut(), None, None),
            Err(CannotUseAbility::NoCharges)
        );
    }
}
