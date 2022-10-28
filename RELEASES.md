# Release Notes

## Version 0.2

### Enhancements

- You can now store and check resource pools (like life, mana or energy) with the `Pool` trait!
  - All of the corresponding ability methods and `AbilityState` have been changed to account for this.
  - Pools have a zero value, a max and a regeneration rate, and are used to track the resource pools of specific actors.  
  - The `Pool` trait has a `Quantity` associated type: this might be used to track the amount stored in a `Pool`, the amount of damage dealt, the life regeneration rate or the mana cost of each ability.
  - For example, you can add `PoolBundle<Mana>` to your entity to track both the `ManaPool` and the `AbilityCosts<A, ManaPool>`.
  - We've included a `LifePool` and `ManaPool` type to get you started; feel free to copy-and-paste to adapt them to your needs.

### Usability

- All methods and functions that returned a bool now return a `Result<(), CannotUseAbility>` which explains why an action failed.
- the `trigger_action` and `action_ready` functions were renamed to `trigger_ability` and `ability_ready`

## Version 0.1

### Enhancements

- You can now store `Cooldowns` and `ActionCharges` on a per-action basis.
  - These new components are now included in the `InputManagerBundle`.
  - Like always, you can choose to use them as a resource instead.
  - Set cooldowns for actions using `CooldownState::set(action, cooldown)` or `CooldownState::new`.
  - Use `Actionlike::ready` with `Actionlike::trigger` as part of your action evaluation!
  - Cooldowns advance whenever `CooldownState::tick` is called (this will happen automatically if you add the plugin).
  - The exact strategy for how charges work for each action can be controlled by the `ReplenishStrategy` and `CooldownStrategy` enums.
