# Release Notes

## Version 0.2

### Usability

- All methods and functions that returned a bool now return a `Result<(), CannotUseAbility>` which explains why an action failed.

## Version 0.1

### Enhancements

- You can now store `Cooldowns` and `ActionCharges` on a per-action basis.
  - These new components are now included in the `InputManagerBundle`.
  - Like always, you can choose to use them as a resource instead.
  - Set cooldowns for actions using `CooldownState::set(action, cooldown)` or `CooldownState::new`.
  - Use `Actionlike::ready` with `Actionlike::trigger` as part of your action evaluation!
  - Cooldowns advance whenever `CooldownState::tick` is called (this will happen automatically if you add the plugin).
  - The exact strategy for how charges work for each action can be controlled by the `ReplenishStrategy` and `CooldownStrategy` enums.
