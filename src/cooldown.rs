//! Cooldowns tick down until actions are ready to be used.

use crate::{
    charges::{ChargeState, Charges},
    Abilitylike, CannotUseAbility,
};

use bevy::utils::Duration;
use bevy::{
    ecs::prelude::{Component, Resource},
    utils::HashMap,
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// The time until each action of type `A` can be used again.
///
/// Each action may be associated with a [`Cooldown`].
/// If it is not, it always be treated as being ready.
///
/// This is typically paired with an [`ActionState`](crate::action_state::ActionState):
/// if the action state is just-pressed (or another triggering condition is met),
/// and the cooldown is ready, then perform the action and trigger the cooldown.
///
/// This type is included as part of the [`InputManagerBundle`](crate::InputManagerBundle),
/// but can also be used as a resource for singleton game objects.
///
///     
/// ```rust
/// use bevy::{utils::Duration, reflect::Reflect};
/// use leafwing_abilities::prelude::*;
/// use leafwing_input_manager::prelude::*;
///
/// #[derive(Actionlike, Abilitylike, Clone, Reflect, PartialEq, Eq, Hash)]
/// enum Action {
///     Run,
///     Jump,
/// }
///
/// let mut action_state = ActionState::<Action>::default();
/// let mut cooldowns = CooldownState::new([(Action::Jump, Cooldown::from_secs(1.))]);
///
/// action_state.press(Action::Jump);
///
/// // This will only perform a limited check; consider using the `Abilitylike::ready` method instead
/// if action_state.just_pressed(Action::Jump) && cooldowns.ready(&Action::Jump).is_ok() {
///    // Actually do the jumping thing here
///    // Remember to actually begin the cooldown if you jumped!
///    cooldowns.trigger(&Action::Jump);
/// }
///
/// // We just jumped, so the cooldown isn't ready yet
/// assert_eq!(cooldowns.ready(&Action::Jump), Err(CannotUseAbility::OnCooldown));
/// ```
#[derive(Resource, Component, Debug, Clone)]
pub struct CooldownState<A: Abilitylike> {
    /// The [`Cooldown`] of each action
    /// If an entry is missing, the action can always be used
    cooldown_map: HashMap<A, Cooldown>,
    /// A shared cooldown between all actions of type `A`.
    ///
    /// No action of type `A` will be ready unless this is ready.
    /// Whenever any cooldown for an action of type `A` is triggered,
    /// this global cooldown is triggered.
    pub global_cooldown: Option<Cooldown>,
    _phantom: PhantomData<A>,
}

impl<A: Abilitylike> Default for CooldownState<A> {
    /// By default, cooldowns are not set.
    fn default() -> Self {
        CooldownState {
            cooldown_map: HashMap::new(),
            global_cooldown: None,
            _phantom: PhantomData,
        }
    }
}

impl<A: Abilitylike> CooldownState<A> {
    /// Creates a new [`CooldownState`] from an iterator of `(cooldown, action)` pairs
    ///
    /// If a [`Cooldown`] is not provided for an action, that action will be treated as if its cooldown is always available.
    ///
    /// To create an empty [`CooldownState`] struct, use the [`Default::default`] method instead.
    ///
    /// # Example
    /// ```rust
    /// use bevy::{input::keyboard::KeyCode, reflect::Reflect};
    /// use leafwing_abilities::cooldown::{Cooldown, CooldownState};
    /// use leafwing_abilities::Abilitylike;
    /// use leafwing_input_manager::Actionlike;
    ///
    /// #[derive(Actionlike, Abilitylike, Clone, Reflect, PartialEq, Eq, Hash)]
    /// enum Action {
    ///     Run,
    ///     Jump,
    ///     Shoot,
    ///     Dash,
    /// }
    ///
    /// let input_map = CooldownState::new([
    ///     (Action::Shoot, Cooldown::from_secs(0.1)),
    ///     (Action::Dash, Cooldown::from_secs(1.0)),
    /// ]);
    /// ```
    #[must_use]
    pub fn new(action_cooldown_pairs: impl IntoIterator<Item = (A, Cooldown)>) -> Self {
        let mut cooldowns = CooldownState::default();
        for (action, cooldown) in action_cooldown_pairs.into_iter() {
            cooldowns.set(action, cooldown);
        }
        cooldowns
    }

    /// Triggers the cooldown of the `action` if it is available to be used.
    ///
    /// This can be paired with [`Cooldowns::ready`],
    /// to check if the action can be used before triggering its cooldown,
    /// or this can be used on its own,
    /// reading the returned [`Result`] to determine if the ability was used.
    #[inline]
    pub fn trigger(&mut self, action: &A) -> Result<(), CannotUseAbility> {
        if let Some(cooldown) = self.get_mut(action) {
            cooldown.trigger()?;
        }

        if let Some(global_cooldown) = self.global_cooldown.as_mut() {
            global_cooldown.trigger()?;
        }

        Ok(())
    }

    /// Can the corresponding `action` be used?
    ///
    /// This will be `Ok` if the underlying [`Cooldown::ready`] call is true,
    /// or if no cooldown is stored for this action.
    #[inline]
    pub fn ready(&self, action: &A) -> Result<(), CannotUseAbility> {
        self.gcd_ready()?;

        if let Some(cooldown) = self.get(action) {
            cooldown.ready()
        } else {
            Ok(())
        }
    }

    /// Has the global cooldown for actions of type `A` expired?
    ///
    /// Returns `Ok(())` if no GCD is set.
    #[inline]
    pub fn gcd_ready(&self) -> Result<(), CannotUseAbility> {
        if let Some(global_cooldown) = self.global_cooldown.as_ref() {
            global_cooldown.ready()
        } else {
            Ok(())
        }
    }

    /// Advances each underlying [`Cooldown`] according to the elapsed `delta_time`.
    ///
    /// When you have a [`Option<Mut<ActionCharges<A>>>`](bevy::ecs::change_detection::Mut),
    /// use `charges.map(|res| res.into_inner())` to convert it to the correct form.
    pub fn tick(&mut self, delta_time: Duration, maybe_charges: Option<&mut ChargeState<A>>) {
        let action_list: Vec<A> = self.cooldown_map.keys().cloned().collect();

        if let Some(charge_state) = maybe_charges {
            for action in &action_list {
                if let Some(ref mut cooldown) = self.get_mut(action) {
                    let charges = charge_state.get_mut(action);
                    cooldown.tick(delta_time, charges);
                }
            }
        } else {
            for action in &action_list {
                if let Some(ref mut cooldown) = self.get_mut(action) {
                    cooldown.tick(delta_time, None);
                }
            }
        }

        if let Some(global_cooldown) = self.global_cooldown.as_mut() {
            global_cooldown.tick(delta_time, None);
        }
    }

    /// The cooldown associated with the specified `action`, if any.
    #[inline]
    #[must_use]
    pub fn get(&self, action: &A) -> Option<&Cooldown> {
        self.cooldown_map.get(action)
    }

    /// A mutable reference to the cooldown associated with the specified `action`, if any.
    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, action: &A) -> Option<&mut Cooldown> {
        self.cooldown_map.get_mut(action)
    }

    /// Set a cooldown for the specified `action`.
    ///
    /// If a cooldown already existed, it will be replaced by a new cooldown with the specified duration.
    #[inline]
    pub fn set(&mut self, action: A, cooldown: Cooldown) -> &mut Self {
        self.cooldown_map.insert(action, cooldown);
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

    /// Returns an iterator of references to the underlying non-[`None`] [`Cooldown`]s
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (&A, &Cooldown)> {
        self.cooldown_map.iter()
    }

    /// Returns an iterator of mutable references to the underlying non-[`None`] [`Cooldown`]s
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&A, &mut Cooldown)> {
        self.cooldown_map.iter_mut()
    }
}

/// A timer-like struct that records the amount of time until an action is available to be used again.
///
/// Cooldowns are typically stored in an [`ActionState`](crate::action_state::ActionState), associated with an action that is to be
/// cooldown-regulated.
///
/// When initialized, cooldowns are always fully available.
///
/// ```rust
/// use bevy::utils::Duration;
/// use leafwing_abilities::cooldown::Cooldown;
/// use leafwing_abilities::CannotUseAbility;
///
/// let mut cooldown = Cooldown::new(Duration::from_secs(3));
/// assert_eq!(cooldown.remaining(), Duration::ZERO);
///
/// cooldown.trigger();
/// assert_eq!(cooldown.remaining(), Duration::from_secs(3));
///
/// cooldown.tick(Duration::from_secs(1), None);
/// assert_eq!(cooldown.ready(), Err(CannotUseAbility::OnCooldown));
///
/// cooldown.tick(Duration::from_secs(5), None);
/// let triggered = cooldown.trigger();
/// assert!(triggered.is_ok());
///
/// cooldown.refresh();
/// assert!(cooldown.ready().is_ok());
/// ```
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Cooldown {
    max_time: Duration,
    /// The amount of time that has elapsed since all [`Charges`](crate::charges::Charges) were fully replenished.
    elapsed_time: Duration,
}

impl Cooldown {
    /// Creates a new [`Cooldown`], which will take `max_time` after it is used until it is ready again.
    ///
    /// # Panics
    ///
    /// The provided max time cannot be [`Duration::ZERO`].
    /// Instead, use [`None`] in the [`Cooldowns`] struct for an action without a cooldown.
    pub fn new(max_time: Duration) -> Cooldown {
        assert!(max_time != Duration::ZERO);

        Cooldown {
            max_time,
            elapsed_time: max_time,
        }
    }

    /// Creates a new [`Cooldown`] with a [`f32`] number of seconds, which will take `max_time` after it is used until it is ready again.
    ///
    /// # Panics
    ///
    /// The provided max time must be greater than 0.
    /// Instead, use [`None`] in the [`CooldownState`] struct for an action without a cooldown.
    pub fn from_secs(max_time: f32) -> Cooldown {
        assert!(max_time > 0.);
        let max_time = Duration::from_secs_f32(max_time);

        Cooldown::new(max_time)
    }

    /// Advance the cooldown by `delta_time`.
    ///
    /// If the elapsed time is enough to reset the cooldown, the number of available charges.
    pub fn tick(&mut self, delta_time: Duration, charges: Option<&mut Charges>) {
        // Don't tick cooldowns when they are fully elapsed
        if self.elapsed_time == self.max_time {
            return;
        }

        assert!(self.max_time != Duration::ZERO);

        if let Some(charges) = charges {
            let total_time = self.elapsed_time.saturating_add(delta_time);

            let total_nanos: u64 = total_time.as_nanos().try_into().unwrap_or(u64::MAX);
            let max_nanos: u64 = self.max_time.as_nanos().try_into().unwrap_or(u64::MAX);

            let n_completed = (total_nanos / max_nanos).try_into().unwrap_or(u8::MAX);
            let extra_time = Duration::from_nanos(total_nanos % max_nanos);

            let excess_completions = charges.add_charges(n_completed);
            if excess_completions == 0 {
                self.elapsed_time =
                    (self.elapsed_time.saturating_add(extra_time)).min(self.max_time);
            } else {
                self.elapsed_time = self.max_time;
            }
        } else {
            self.elapsed_time = self
                .elapsed_time
                .saturating_add(delta_time)
                .min(self.max_time);
        }
    }

    /// Is this action ready to be used?
    ///
    /// This will be true if and only if at least one charge is available.
    /// For cooldowns without charges, this will be true if `time_remaining` is [`Duration::Zero`].
    pub fn ready(&self) -> Result<(), CannotUseAbility> {
        match self.elapsed_time >= self.max_time {
            true => Ok(()),
            false => Err(CannotUseAbility::OnCooldown),
        }
    }

    /// Refreshes the cooldown, causing the underlying action to be ready to use immediately.
    ///
    /// If this cooldown has charges, the number of available charges is increased by one (but the point within the cycle is unchanged).
    #[inline]
    pub fn refresh(&mut self) {
        self.elapsed_time = self.max_time
    }

    /// Use the underlying cooldown if and only if it is ready, resetting the cooldown to its maximum value.
    ///
    /// If this cooldown has multiple charges, only one will be consumed.
    ///
    /// Returns a result indicating whether the cooldown was ready.
    /// If the cooldown was not ready, [`CannotUseAbility::OnCooldown`] is returned and this call has no effect.
    #[inline]
    pub fn trigger(&mut self) -> Result<(), CannotUseAbility> {
        self.ready()?;
        self.elapsed_time = Duration::ZERO;

        Ok(())
    }

    /// Returns the time that it will take for this action to be ready to use again after being triggered.
    #[inline]
    pub fn max_time(&self) -> Duration {
        self.max_time
    }

    /// Sets the time that it will take for this action to be ready to use again after being triggered.
    ///
    /// If the current time remaining is greater than the new max time, it will be clamped to the `max_time`.
    ///
    /// # Panics
    ///
    /// The provided max time cannot be [`Duration::ZERO`].
    /// Instead, use [`None`] in the [`Cooldowns`] struct for an action without a cooldown.
    #[inline]
    pub fn set_max_time(&mut self, max_time: Duration) {
        assert!(max_time != Duration::ZERO);

        self.max_time = max_time;
        self.elapsed_time = self.elapsed_time.min(max_time);
    }

    /// Returns the time that has passed since the cooldown was triggered.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.elapsed_time
    }

    /// Sets the time that has passed since the cooldown was triggered.
    ///
    /// This will always be clamped between [`Duration::ZERO`] and the `max_time` of this cooldown.
    #[inline]
    pub fn set_elapsed(&mut self, elapsed_time: Duration) {
        self.elapsed_time = elapsed_time.clamp(Duration::ZERO, self.max_time);
    }

    /// Returns the time remaining until the next charge is ready.
    ///
    /// When a cooldown is fully charged, this will return [`Duration::ZERO`].
    #[inline]
    pub fn remaining(&self) -> Duration {
        self.max_time.saturating_sub(self.elapsed_time)
    }

    /// Sets the time remaining until the next charge is ready.
    ///
    /// This will always be clamped between [`Duration::ZERO`] and the `max_time` of this cooldown.
    #[inline]
    pub fn set_remaining(&mut self, time_remaining: Duration) {
        self.elapsed_time = self
            .max_time
            .saturating_sub(time_remaining.clamp(Duration::ZERO, self.max_time));
    }
}

#[cfg(test)]
mod tick_tests {
    use super::*;

    #[test]
    #[should_panic]
    fn zero_duration_cooldown_cannot_be_constructed() {
        Cooldown::new(Duration::ZERO);
    }

    #[test]
    fn tick_has_no_effect_on_fresh_cooldown() {
        let cooldown = Cooldown::from_secs(1.);
        let mut cloned_cooldown = cooldown.clone();
        cloned_cooldown.tick(Duration::from_secs_f32(1.234), None);
        assert_eq!(cooldown, cloned_cooldown);
    }

    #[test]
    fn cooldowns_start_ready() {
        let cooldown = Cooldown::from_secs(1.);
        assert!(cooldown.ready().is_ok());
    }

    #[test]
    fn cooldowns_are_ready_when_refreshed() {
        let mut cooldown = Cooldown::from_secs(1.);
        assert!(cooldown.ready().is_ok());
        let _ = cooldown.trigger();
        assert_eq!(cooldown.ready(), Err(CannotUseAbility::OnCooldown));
        cooldown.refresh();
        assert!(cooldown.ready().is_ok());
    }

    #[test]
    fn ticking_changes_cooldown() {
        let cooldown = Cooldown::new(Duration::from_millis(1000));
        let mut cloned_cooldown = cooldown.clone();
        let _ = cloned_cooldown.trigger();
        assert!(cooldown != cloned_cooldown);

        cloned_cooldown.tick(Duration::from_millis(123), None);
        assert!(cooldown != cloned_cooldown);
    }

    #[test]
    fn cooldowns_reset_after_being_ticked() {
        let mut cooldown = Cooldown::from_secs(1.);
        let _ = cooldown.trigger();
        assert_eq!(cooldown.ready(), Err(CannotUseAbility::OnCooldown));

        cooldown.tick(Duration::from_secs(3), None);
        assert!(cooldown.ready().is_ok());
    }

    #[test]
    fn time_remaining_on_fresh_cooldown_is_zero() {
        let cooldown = Cooldown::from_secs(1.);
        assert_eq!(cooldown.remaining(), Duration::ZERO);
    }
}
