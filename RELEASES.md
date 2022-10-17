# Release Notes

## Version 0.1

### Enhancements

- You can now store `Cooldowns` and `ActionCharges` on a per-action basis.
  - These new components are now included in the `InputManagerBundle`.
  - Like always, you can choose to use them as a resource instead.
  - Set cooldowns for actions using `Cooldowns::set(action, cooldown)` or `Cooldowns::new`.
  - Use `Actionlike::ready` with `Actionlike::trigger` as part of your action evaluation!
  - Cooldowns advance whenever `Cooldowns::tick` is called (this will happen automatically if you add the plugin).
  - The exact strategy for how charges work for each action can be controlled by the `ReplenishStrategy` and `CooldownStrategy` enums.
