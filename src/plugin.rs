//! Contains main plugin exported by this crate.

use crate::Abilitylike;
use bevy::ecs::prelude::*;
use core::marker::PhantomData;

use bevy::app::{App, Plugin, PreUpdate};
use leafwing_input_manager::plugin::InputManagerSystem;

/// A [`Plugin`] that collects [`Input`](bevy::input::Input) from disparate sources, producing an [`ActionState`](crate::action_state::ActionState) that can be conveniently checked
///
/// This plugin needs to be passed in an [`Actionlike`] enum type that you've created for your game.
/// Each variant represents a "virtual button" whose state is stored in an [`ActionState`](crate::action_state::ActionState) struct.
///
/// Each [`InputManagerBundle`](crate::InputManagerBundle) contains:
///  - an [`InputMap`](crate::input_map::InputMap) component, which stores an entity-specific mapping between the assorted input streams and an internal representation of "actions"
///  - an [`ActionState`](crate::action_state::ActionState) component, which stores the current input state for that entity in an source-agnostic fashion
///
/// If you have more than one distinct type of action (e.g. menu actions, camera actions and player actions), consider creating multiple `Actionlike` enums
/// and adding a copy of this plugin for each `Actionlike` type.
///  
/// ## Systems
///
/// **WARNING:** These systems run during [`CoreStage::PreUpdate`].
/// If you have systems that care about inputs and actions that also run during this stage,
/// you must define an ordering between your systems or behavior will be very erratic.
/// The stable labels for these systems are available under [`InputManagerSystem`] enum.
///
/// Complete list:
///
/// - [`tick_action_state`](crate::systems::tick_action_state), which resets the `pressed` and `just_pressed` fields of the [`ActionState`](crate::action_state::ActionState) each frame
///     - labeled [`InputManagerSystem::Reset`]
/// - [`update_action_state`](crate::systems::update_action_state), which collects [`Input`](bevy::input::Input) resources to update the [`ActionState`](crate::action_state::ActionState)
///     - labeled [`InputManagerSystem::Update`]
/// - [`update_action_state_from_interaction`](crate::systems::update_action_state_from_interaction), for triggering actions from buttons
///    - powers the [`ActionStateDriver`](crate::action_state::ActionStateDriver) component baseod on an [`Interaction`](bevy::ui::Interaction) component
///    - labeled [`InputManagerSystem::Update`]
/// - [`release_on_disable`](crate::systems::release_on_disable), which resets action states when [`ToggleActions`] is flipped, to avoid persistent presses.
pub struct AbilityPlugin<A: Abilitylike> {
    _phantom: PhantomData<A>,
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
                .in_set(InputManagerSystem::Tick)
                .before(InputManagerSystem::Update),
        );
    }
}
