# About

A fully-featured set of tools for managing abilities in [Bevy](https://bevyengine.org/).
This crate is meant to be used with [Leafwing Input Manager](https://github.com/leafwing-studios/leafwing-input-manager), which converts inputs into actions.

Some of those actions will be abilities!
Abilities are intended for gameplay use, and follow complex but relatively standardized logic about how they might be used.

```rust
use leafwing_input_manager::prelude::*;
use leafwing_abilities::prelude::*;
use bevy::prelude::*;

// We're modelling https://leagueoflegends.fandom.com/wiki/Zyra/LoL
// to show off this crate's features!
#[derive(Actionlike, Abilitylike, Clone)]
pub enum ZyraAbilties {
  GardenOfThorns,
  DeadlySpines,
  RampantGrowth,
  GraspingRoots,
  Stranglethorns,
}

impl ZyraAbility {
  fn input_map() -> InputMap<ZyraAbility> {
    use ZyraAbility::*;

    // We can use this `new` idiom, which accepts an iterator of pairs
    InputMap::new([
      (KeyCode::Q, DeadlySpines),
      (KeyCode::W, RampantGrowth),
      (KeyCode::E, GraspingRoots),
      (KeyCode::R, Stranglethorns),
    ])
  }

  // This match pattern is super useful to be sure you've defined an attribute for every variant
  fn cooldown(&self) -> Cooldown {
    use ZyraAbility::*;

    let seconds: f32 = match *self {
      GardenOfThorns => 13.
      DeadlySpines => 7.
      RampantGrowth => 18.
      GraspingRoots => 12.
      Stranglethorns => 110.
    };
    
    Cooldown::from_secs(seconds)
  }

  fn cooldowns() -> CooldownState<ZyraAbility> {
    use ZyraAbility::*;

    let mut cooldowns = CooldownState::default();

    // Now, we can loop over all the variants to populate our struct
    for ability in ZyraAbility::variants() {
      cooldowns.set(ability, ability.cooldown());
    }

    cooldowns
  }

  fn charges() -> ChargeState<ZyraAbility> {
    // The builder API can be very convenient when you only need to set a couple of values
    ChargeState::default().set(ZyraAbility::RampantGrowth, Charges::replenish_one(2))
  }
}

/// Marker component for this champion
#[derive(Component)]
struct Zyra;

#[derive(Bundle)]
struct ZyraBundle {
  champion: Zyra,
  #[bundle]
  input_manager_bundle: InputManagerBundle<ZyraAction>,
  #[bundle]
  abilities_bundle: AbilitiesBundle<ZyraAction>,
}

impl Default for ZyraBundle {
  fn default() -> Self {
    ZyraBundle {
      champion: Zyra,
      input_manager_bundle: InputManagerBundle {
        input_map: ZyraAbility::input_map(),
        ..default()
      }
      has_abilities_bundle: AbilitiesBundle {
        cooldowns: ZyraAbility::cooldowns(),
        charges: ZyraAbilties::charges(),
      }
    }
  }
}
```

## Features

- track and automatically tick cooldowns
- store multiple charges of abilities

Planned:

- resource management (health, mana, energy etc)
- damage
- cast times
- range checking
