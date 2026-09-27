//! Contains main plugin exported by this crate.

use crate::Abilitylike;
use bevy::ecs::prelude::*;
use core::marker::PhantomData;

use bevy::app::{App, Plugin, PreUpdate};
use leafwing_input_manager::plugin::InputManagerSystem;

/// A [`Plugin`] that advances the abilities of type `A`.
///
/// This plugin ticks down the [`CooldownState`](crate::cooldown::CooldownState) and
/// [`ChargeState`](crate::charges::ChargeState) components of every entity with abilities of type `A`,
/// replenishing charges and refreshing cooldowns as they expire.
///
/// This plugin needs to be passed in an [`Abilitylike`] enum type that you've created for your game.
/// If you have more than one distinct type of action (e.g. menu actions, camera actions and player actions),
/// consider creating multiple `Abilitylike` enums and adding a copy of this plugin for each `Abilitylike` type.
///
/// ## Systems
///
/// The plugin adds [`tick_cooldowns`](crate::systems::tick_cooldowns) to
/// [`PreUpdate`], in the [`AbilitySystem::TickCooldowns`] set.
/// It is ordered before [`InputManagerSystem::Update`],
/// so cooldowns are advanced before action states are updated for the frame.
///
/// **WARNING:** These systems run during [`PreUpdate`].
/// If you have systems that care about inputs and actions that also run during this stage,
/// you must define an ordering between your systems or behavior will be very erratic.
pub struct AbilityPlugin<A: Abilitylike> {
    _phantom: PhantomData<A>,
}

/// System sets provided by [`leafwing_abilities`](crate).
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AbilitySystem {
    /// Updates the cooldowns of all abilities,
    /// including their charges and global cooldowns if applicable.
    TickCooldowns,
}

// Deriving default induces an undesired bound on the generic
impl<A: Abilitylike> Default for AbilityPlugin<A> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<A: Abilitylike> Plugin for AbilityPlugin<A> {
    fn build(&self, app: &mut App) {
        use crate::systems::*;

        // Systems
        app.add_systems(
            PreUpdate,
            tick_cooldowns::<A>
                .in_set(AbilitySystem::TickCooldowns)
                .in_set(InputManagerSystem::Tick)
                .before(InputManagerSystem::Update),
        );
    }
}
