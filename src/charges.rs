//! Charges are "uses of an action".
//! Actions may only be used if at least one charge is available.
//! Unlike pools, charges are not shared across abilities.

use bevy::{
    ecs::prelude::{Component, Resource},
    reflect::Reflect,
};
use std::{fmt::Display, marker::PhantomData};

use crate::{Abilitylike, CannotUseAbility};
use std::collections::HashMap;

/// A component / resource that stores the [`Charges`] for each [`Abilitylike`] action of type `A`.
///
/// If [`Charges`] is set for an actions, it is only [`Abilitylike::ready`] when at least one charge is available.
///
/// ```rust
/// use bevy::reflect::Reflect;
/// use leafwing_abilities::prelude::*;
/// use leafwing_abilities::premade_pools::mana::{Mana, ManaPool};
/// use leafwing_input_manager::Actionlike;
///
/// #[derive(Actionlike, Abilitylike, Clone, Reflect, Hash, PartialEq, Eq)]
/// enum Action {
///     // Neither cooldowns nor charges
///     Move,
///     // Double jump: 2 charges, no cooldowns
///     Jump,
///     // Simple cooldown
///     Dash,
///     // Cooldowns and charges, replenishing one at a time
///     Spell,
/// }
///
/// impl Action {
///     fn charges() -> ChargeState<Action> {
///         // You can either use the builder pattern or the `new` init for both cooldowns and charges
///         // The differences are largely aesthetic.
///         ChargeState::default()
///             // Double jump!
///             .set(Action::Jump, Charges::replenish_all(2))
///             // Store up to 3 spells at once
///             .set(Action::Spell, Charges::replenish_one(3))
///             .build()
///     }
///
///     fn cooldowns() -> CooldownState<Action> {
///         // Omitted cooldowns and charges will cause the action to be treated as if it always had available cooldowns / charges to use.
///         CooldownState::new([
///             (Action::Dash, Cooldown::from_secs(2.)),
///             (Action::Spell, Cooldown::from_secs(4.5)),
///         ])
///     }
///
///     fn mana_costs() -> AbilityCosts<Action, ManaPool> {
///         // Provide the Pool::Quantity value when setting costs
///         AbilityCosts::new([
///             (Action::Spell, Mana(10.)),
///         ])
///     }
/// }
///
/// // In a real game you'd spawn a bundle with the appropriate components.
/// let mut abilities_bundle = AbilitiesBundle {
///     cooldowns: Action::cooldowns(),
///     charges: Action::charges(),
///     ..Default::default()
/// };
///
/// // You can also define resource pools using a separate bundle.
/// // Typically, you'll want to nest both of these bundles under a custom Bundle type for your characters.
/// let mut mana_bundle = PoolBundle {
///     // Max mana of 100., regen rate of 1.
///     pool: ManaPool::new(Mana(100.0), Mana(100.0), Mana(1.0)),
///     ability_costs: Action::mana_costs(),     
/// };
///
/// // Then, you can check if an action is ready to be used.
/// // Consider using the `AbilityState` `WorldQuery` type instead for convenience!
/// if Action::Spell.ready(&abilities_bundle.charges, &abilities_bundle.cooldowns, Some(&mana_bundle.pool), Some(&mana_bundle.ability_costs)).is_ok() {
///     // When you use an action, remember to trigger it!
///     Action::Spell.trigger(&mut abilities_bundle.charges, &mut abilities_bundle.cooldowns, Some(&mut mana_bundle.pool), Some(&mut mana_bundle.ability_costs));
/// }
/// ```
#[derive(Resource, Component, Clone, PartialEq, Eq, Debug, Reflect)]
pub struct ChargeState<A: Abilitylike> {
    /// The underlying [`Charges`].
    charges_map: HashMap<A, Charges>,
    #[reflect(ignore)]
    _phantom: PhantomData<A>,
}

impl<A: Abilitylike> Default for ChargeState<A> {
    fn default() -> Self {
        ChargeState {
            charges_map: HashMap::new(),
            _phantom: PhantomData,
        }
    }
}

/// Stores how many times an action can be used.
///
/// Charges refresh when [`Charges::refresh`] is called manually,
/// or when the corresponding cooldown expires (if the [`InputManagerPlugin`](crate::plugin::InputManagerPlugin) is added).
#[derive(Clone, Default, PartialEq, Eq, Debug, Reflect)]
pub struct Charges {
    current: u8,
    max: u8,
    /// What should happen when the charges are refreshed?
    pub replenish_strat: ReplenishStrategy,
    /// How should the corresponding [`Cooldown`](crate::cooldown::Cooldown) interact with these charges?
    pub cooldown_strat: CooldownStrategy,
}

impl Display for Charges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.current, self.max)
    }
}

/// What happens when [`Charges`] are replenished?
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum ReplenishStrategy {
    /// A single charge will be recovered.
    ///
    /// Usually paired with [`CooldownStrategy::ConstantlyRefresh`].
    #[default]
    OneAtATime,
    /// All charges will be recovered.
    ///
    /// Usually paired with [`CooldownStrategy::RefreshWhenEmpty`].
    AllAtOnce,
}

/// How do these charges replenish when cooldowns are refreshed?
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum CooldownStrategy {
    /// Cooldowns refresh will have no effect on the charges.
    Ignore,
    /// Cooldowns will replenish charges whenever the current charges are less than the max.
    ///
    /// Usually paired with [`ReplenishStrategy::OneAtATime`].
    #[default]
    ConstantlyRefresh,
    /// Cooldowns will only replenish charges when 0 charges are available.
    ///
    /// Usually paired with [`ReplenishStrategy::AllAtOnce`].
    RefreshWhenEmpty,
}

impl<A: Abilitylike> ChargeState<A> {
    /// Creates a new [`ChargeState`] from an iterator of `(charges, action)` pairs
    ///
    /// If a [`Charges`] is not provided for an action, that action will be treated as if a charge was always available.
    ///
    /// To create an empty [`ChargeState`] struct, use the [`Default::default`] method instead.
    ///
    /// # Example
    /// ```rust
    /// use bevy::{input::keyboard::KeyCode, reflect::Reflect};
    /// use leafwing_abilities::prelude::*;
    /// use leafwing_input_manager::Actionlike;
    ///
    /// #[derive(Actionlike, Abilitylike, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    ///     Shoot,
    ///     Dash,
    /// }
    ///
    /// let charge_state = ChargeState::new([
    ///     (Action::Shoot, Charges::replenish_all(6)),
    ///     (Action::Dash, Charges::replenish_one(2)),
    /// ]);
    /// ```
    #[must_use]
    pub fn new(action_chargestate_pairs: impl IntoIterator<Item = (A, Charges)>) -> Self {
        let mut charge_state = ChargeState::default();
        for (action, charges) in action_chargestate_pairs.into_iter() {
            charge_state.set(action, charges);
        }
        charge_state
    }

    /// Is at least one charge available for `action`?
    ///
    /// Returns `true` if the underlying [`Charges`] is [`None`].
    #[inline]
    #[must_use]
    pub fn available(&self, action: A) -> bool {
        if let Some(charges) = self.get(action) {
            charges.available()
        } else {
            true
        }
    }

    /// Spends one charge for `action` if able.
    ///
    /// Returns a boolean indicating whether a charge was available.
    /// If no charges are available, `false` is returned and this call has no effect.
    ///
    /// Returns `true` if the underlying [`Charges`] is [`None`].
    #[inline]
    pub fn expend(&mut self, action: A) -> Result<(), CannotUseAbility> {
        if let Some(charges) = self.get_mut(action) {
            charges.expend()
        } else {
            Ok(())
        }
    }

    /// Replenishes charges of `action`, up to its max charges.
    ///
    /// The exact effect is determined by the [`Charges`]'s [`ReplenishStrategy`].
    /// If the `action` is not associated with a [`Charges`], this has no effect.
    #[inline]
    pub fn replenish(&mut self, action: A) {
        if let Some(charges) = self.get_mut(action) {
            charges.replenish();
        }
    }

    /// Returns a reference to the underlying [`Charges`] for `action`, if set.
    #[inline]
    #[must_use]
    pub fn get(&self, action: A) -> Option<&Charges> {
        self.charges_map.get(&action)
    }

    /// Returns a mutable reference to the underlying [`Charges`] for `action`, if set.
    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, action: A) -> Option<&mut Charges> {
        self.charges_map.get_mut(&action)
    }

    /// Sets the underlying [`Charges`] for `action` to the provided value.
    ///
    /// Unless you're building a new [`ChargeState`] struct, you likely want to use [`Self::get_mut`].
    #[inline]
    pub fn set(&mut self, action: A, charges: Charges) -> &mut Self {
        self.charges_map.insert(action, charges);

        self
    }

    /// Collects a `&mut Self` into a `Self`.
    ///
    /// Used to conclude the builder pattern. Actually just calls `self.clone()`.
    #[inline]
    #[must_use]
    pub fn build(&mut self) -> Self {
        self.clone()
    }

    /// Returns an iterator of references to the underlying non-[`None`] [`Charges`]
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Charges> {
        self.charges_map.values()
    }

    /// Returns an iterator of mutable references to the underlying non-[`None`] [`Charges`]
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Charges> {
        self.charges_map.values_mut()
    }
}

impl Charges {
    /// Creates a new [`Charges`], which can be expended `max_charges` times before needing to be replenished.
    ///
    /// The current charges will be set to the max charges by default.
    #[inline]
    #[must_use]
    pub fn new(
        max_charges: u8,
        replenish_strat: ReplenishStrategy,
        cooldown_strat: CooldownStrategy,
    ) -> Charges {
        Charges {
            current: max_charges,
            max: max_charges,
            replenish_strat,
            cooldown_strat,
        }
    }

    /// Creates a new [`Charges`] with [`ReplenishStrategy::OneAtATime`] and [`CooldownStrategy::Ignore`].
    pub fn simple(max_charges: u8) -> Charges {
        Charges {
            current: max_charges,
            max: max_charges,
            replenish_strat: ReplenishStrategy::OneAtATime,
            cooldown_strat: CooldownStrategy::Ignore,
        }
    }

    /// Creates a new [`Charges`] with [`ReplenishStrategy::AllAtOnce`] and [`CooldownStrategy::Ignore`].
    pub fn ammo(max_charges: u8) -> Charges {
        Charges {
            current: max_charges,
            max: max_charges,
            replenish_strat: ReplenishStrategy::AllAtOnce,
            cooldown_strat: CooldownStrategy::Ignore,
        }
    }

    /// Creates a new [`Charges`] with [`ReplenishStrategy::OneAtATime`] and [`CooldownStrategy::ConstantlyRefresh`].
    pub fn replenish_one(max_charges: u8) -> Charges {
        Charges {
            current: max_charges,
            max: max_charges,
            replenish_strat: ReplenishStrategy::OneAtATime,
            cooldown_strat: CooldownStrategy::ConstantlyRefresh,
        }
    }

    /// Creates a new [`Charges`] with [`ReplenishStrategy::AllAtOnce`] and [`CooldownStrategy::RefreshWhenEmpty`].
    pub fn replenish_all(max_charges: u8) -> Charges {
        Charges {
            current: max_charges,
            max: max_charges,
            replenish_strat: ReplenishStrategy::AllAtOnce,
            cooldown_strat: CooldownStrategy::RefreshWhenEmpty,
        }
    }

    /// The current number of available charges
    #[inline]
    #[must_use]
    pub fn charges(&self) -> u8 {
        self.current
    }

    /// The maximum number of available charges
    #[inline]
    #[must_use]
    pub fn max_charges(&self) -> u8 {
        self.max
    }

    /// Adds `charges` to the current number of available charges
    ///
    /// This will never exceed the maximum number of charges.
    /// Returns the number of excess charges.
    #[inline]
    #[must_use]
    pub fn add_charges(&mut self, charges: u8) -> u8 {
        let new_total = self.current.saturating_add(charges);

        let excess = new_total.saturating_sub(self.max);
        self.current = new_total.min(self.max);
        excess
    }

    /// Set the current number of available charges
    ///
    /// This will never exceed the maximum number of charges.
    /// Returns the number of excess charges.
    #[inline]
    pub fn set_charges(&mut self, charges: u8) -> u8 {
        let excess = charges.saturating_sub(self.max);
        self.current = charges.min(self.max);
        excess
    }

    /// Set the maximmum number of available charges
    ///
    /// If the number of charges available is greater than this number, it will be reduced to the new cap.
    #[inline]
    pub fn set_max_charges(&mut self, max_charges: u8) {
        self.max = max_charges;
        self.current = self.current.min(self.max);
    }

    /// Is at least one charge available?
    #[inline]
    #[must_use]
    pub fn available(&self) -> bool {
        self.current > 0
    }

    /// Spends one charge for `action` if able.
    ///
    /// Returns a [`Result`] indicating whether a charge was available.
    /// If no charges are available, [`CannotUseAbility::NoCharges`] is returned and this call has no effect.
    #[inline]
    pub fn expend(&mut self) -> Result<(), CannotUseAbility> {
        if self.current == 0 {
            return Err(CannotUseAbility::NoCharges);
        }

        self.current = self.current.saturating_sub(1);
        Ok(())
    }

    /// Replenishes charges of `action`, up to its max charges.
    ///
    /// The exact effect is determined by the [`ReplenishStrategy`] for this struct.
    #[inline]
    pub fn replenish(&mut self) {
        let charges_to_add = match self.replenish_strat {
            ReplenishStrategy::OneAtATime => 1,
            ReplenishStrategy::AllAtOnce => self.max,
        };

        // We don't care about overflowing our charges here.
        let _ = self.add_charges(charges_to_add);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charges_start_full() {
        let charges = Charges::simple(3);
        assert_eq!(charges.charges(), 3);
        assert_eq!(charges.max_charges(), 3);
    }

    #[test]
    fn charges_available() {
        let mut charges = Charges::simple(3);
        assert!(charges.available());
        charges.set_charges(1);
        assert!(charges.available());
        charges.set_charges(0);
        assert!(!charges.available());
    }

    #[test]
    fn charges_deplete() {
        let mut charges = Charges::simple(2);
        charges.expend().unwrap();
        assert_eq!(charges.charges(), 1);
        charges.expend().unwrap();
        assert_eq!(charges.charges(), 0);
        assert_eq!(charges.expend(), Err(CannotUseAbility::NoCharges));
        assert_eq!(charges.charges(), 0);
    }

    #[test]
    fn charges_replenish_one_at_a_time() {
        let mut charges = Charges::replenish_one(3);
        charges.set_charges(0);
        assert_eq!(charges.charges(), 0);
        charges.replenish();
        assert_eq!(charges.charges(), 1);
        charges.replenish();
        assert_eq!(charges.charges(), 2);
        charges.replenish();
        assert_eq!(charges.charges(), 3);
        charges.replenish();
        assert_eq!(charges.charges(), 3);
    }

    #[test]
    fn charges_replenish_all_at_once() {
        let mut charges = Charges::replenish_all(3);
        charges.set_charges(0);
        assert_eq!(charges.charges(), 0);
        charges.replenish();
        assert_eq!(charges.charges(), 3);
    }
}
