# Release Notes

## Version 0.9

## Dependencies (0.9)

- now support `leafwing-input-manager` 0.15

## Usability (0.9)

- all types provided by this library are now `Reflect`
- removed `ToggleActions`: this functionality no longer makes sense with changes to how LWIM disables actions. Use run conditions directly on the new `AbilitySystem::TickCooldowns` system set
- the associated type `Pool::Quantity` no longer needs to be able to be multiplied and divided by f32s to ease working with integer-based resource pools
  - in exchange, the `RegeneratingPool::regenerate` method no longer has a default implementation
- to better support working with multiple resource pools for a single `Abilitylike`:
  - `ready_no_cost` and `trigger_no_cost` have been added to `Abilitylike`
  - when working with multiple resource pools, you should pass in `NullPool` as the type argument for `AbilityState`
- `Default` is now implemented for `Charges` and `Cooldown`

## Bugs (0.9)

- `Actionlike::trigger` and friends now expends resource costs correctly
  - if you were working around this bug, remember to remove your workaround to avoid double-spending!

## Version 0.8

- now supports Bevy 0.14

## Version 0.7

### Dependencies (0.7)

- now supports Bevy 0.13

## Version 0.6

### Dependencies (0.6)

- now supports Bevy 0.12

### Documentation (0.6)

- fixed several typos (`@striezel`)
- improved the documentation for `Pool::replenish`

### Usability (0.6)

- removed the required `new` method from the `Pool` trait: this method was overly restrictive, and prevented the construction of more complex pools with custom initialization parameters
  - `LifePool::new` and `ManaPool::new` methods have been added to the premade pools: do similarly for your own `Pool` types
- the `Pool::ZERO` associated constant has been renamed to the clearer `Pool::MIN`.
  - the `MaxPoolLessThanZero` error type has been renamed to `MaxPoolLessThanMin` to match.
- the `Pool` trait has been split in two, with the regeneration-specific mechanics handled in `RegeneratingPool`, to make the construction of non-regenerating pools much more intuitive
- added the `Pool::is_empty` and `Pool::is_full` helper methods to the `Pool` trait
- added `Add`, `Sub`, `AddAssign` and `SubAssign` implementations to the premade `Life` and `Mana` types and their corresponding pools
- added the `Display` trait to `Life`, `Mana`, `LifePool` and `ManaPool`
- removed the useless `AbilityPlugin::server()` plugin creation method

## Version 0.5

### Dependencies (0.5)

- now supports Bevy 0.11

## Version 0.4

### Dependencies (0.4)

- now supports Bevy 0.10

### Usability (0.4)

- the premade `LifePool` and `ManaPool` types now implement the `Resource` trait.
- the premade `Life` and `Mana` types now implement `Mul<T> for f32`, allowing you to have commutative multiplication

## Version 0.3

### Dependencies (0.3)

- now supports Bevy 0.9

## Version 0.2

### Enhancements (0.2)

- You can now store and check resource pools (like life, mana or energy) with the `Pool` trait!
  - All of the corresponding ability methods and `AbilityState` have been changed to account for this.
  - Pools have a zero value, a max and a regeneration rate, and are used to track the resource pools of specific actors.  
  - The `Pool` trait has a `Quantity` associated type: this might be used to track the amount stored in a `Pool`, the amount of damage dealt, the life regeneration rate or the mana cost of each ability.
  - For example, you can add `PoolBundle<Mana>` to your entity to track both the `ManaPool` and the `AbilityCosts<A, ManaPool>`.
  - We've included a `LifePool` and `ManaPool` type to get you started; feel free to copy-and-paste to adapt them to your needs.

### Usability (0.2)

- All methods and functions that returned a bool now return a `Result<(), CannotUseAbility>` which explains why an action failed.
- the `trigger_action` and `action_ready` functions were renamed to `trigger_ability` and `ability_ready`

## Version 0.1

### Enhancements (0.1)

- You can now store `Cooldowns` and `ActionCharges` on a per-action basis.
  - These new components are now included in the `InputManagerBundle`.
  - Like always, you can choose to use them as a resource instead.
  - Set cooldowns for actions using `CooldownState::set(action, cooldown)` or `CooldownState::new`.
  - Use `Actionlike::ready` with `Actionlike::trigger` as part of your action evaluation!
  - Cooldowns advance whenever `CooldownState::tick` is called (this will happen automatically if you add the plugin).
  - The exact strategy for how charges work for each action can be controlled by the `ReplenishStrategy` and `CooldownStrategy` enums.
