// BLOCKED: these tests should set the time manually.
// Requires https://github.com/bevyengine/bevy/issues/6146 to do so.

use bevy::input::InputPlugin;
use bevy::prelude::*;
use core::time::Duration;
use leafwing_abilities::prelude::*;
use leafwing_input_manager::prelude::*;

use std::thread::sleep;

#[derive(Actionlike, Reflect, Abilitylike, Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Action {
    NoCooldown,
    Short,
    Long,
}

impl Action {
    /// You could use the `strum` crate to derive this automatically!
    fn variants() -> impl Iterator<Item = Action> {
        use Action::*;
        [NoCooldown, Short, Long].iter().copied()
    }

    fn cooldown(&self) -> Option<Cooldown> {
        match self {
            Action::NoCooldown => None,
            Action::Short => Some(Cooldown::from_secs(0.1)),
            Action::Long => Some(Cooldown::from_secs(1.)),
        }
    }

    fn cooldowns() -> CooldownState<Action> {
        let mut cd = CooldownState::default();
        for action in Action::variants() {
            if let Some(cooldown) = action.cooldown() {
                cd.set(action, cooldown);
            }
        }

        cd
    }
}

fn spawn(mut commands: Commands) {
    commands.spawn(AbilitiesBundle {
        cooldowns: Action::cooldowns(),
        ..default()
    });
}

#[test]
fn cooldowns_on_entity() {
    use Action::*;

    let mut app = App::new();
    app.add_plugins(AbilityPlugin::<Action>::default())
        .add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .add_systems(Startup, spawn);

    // Spawn entities
    app.update();

    // Cooldown start ready
    let mut query_state = app.world_mut().query::<&mut CooldownState<Action>>();
    let mut cooldowns: Mut<CooldownState<Action>> =
        query_state.single_mut(app.world_mut()).unwrap();
    for action in Action::variants() {
        assert!(cooldowns.ready(&action).is_ok());
        // Trigger all the cooldowns once
        let _ = cooldowns.trigger(&action);
    }

    app.update();

    // No waiting
    let mut query_state = app.world_mut().query::<&CooldownState<Action>>();
    let cooldowns: &CooldownState<Action> = query_state.single(app.world()).unwrap();
    assert!(cooldowns.ready(&NoCooldown).is_ok());
    assert_eq!(cooldowns.ready(&Short), Err(CannotUseAbility::OnCooldown));
    assert_eq!(cooldowns.ready(&Long), Err(CannotUseAbility::OnCooldown));

    sleep(Duration::from_secs_f32(0.2));
    app.update();

    // Short wait
    let mut query_state = app.world_mut().query::<&CooldownState<Action>>();
    let cooldowns: &CooldownState<Action> = query_state.single(&app.world()).unwrap();
    assert!(cooldowns.ready(&NoCooldown).is_ok());
    assert!(cooldowns.ready(&Short).is_ok());
    assert_eq!(cooldowns.ready(&Long), Err(CannotUseAbility::OnCooldown));
}

#[test]
fn cooldowns_in_resource() {
    use Action::*;

    let mut app = App::new();
    app.add_plugins(AbilityPlugin::<Action>::default())
        .add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .insert_resource(Action::cooldowns());

    // Cooldown start ready
    let mut cooldowns: Mut<CooldownState<Action>> = app.world_mut().resource_mut();
    for action in Action::variants() {
        assert!(cooldowns.ready(&action).is_ok());
        let _ = cooldowns.trigger(&action);
    }

    app.update();

    // No waiting
    let cooldowns: &CooldownState<Action> = app.world().resource();
    assert!(cooldowns.ready(&NoCooldown).is_ok());
    assert_eq!(cooldowns.ready(&Short), Err(CannotUseAbility::OnCooldown));
    assert_eq!(cooldowns.ready(&Long), Err(CannotUseAbility::OnCooldown));

    sleep(Duration::from_secs_f32(0.2));
    app.update();

    // Short wait
    let cooldowns: &CooldownState<Action> = app.world().resource();
    assert!(cooldowns.ready(&NoCooldown).is_ok());
    assert!(cooldowns.ready(&Short).is_ok());
    assert_eq!(cooldowns.ready(&Long), Err(CannotUseAbility::OnCooldown));
}

#[test]
fn global_cooldowns_tick() {
    let mut app = App::new();
    app.add_plugins(AbilityPlugin::<Action>::default())
        .add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .insert_resource(Action::cooldowns());

    let mut cooldowns: Mut<CooldownState<Action>> = app.world_mut().resource_mut();
    let initial_gcd = Some(Cooldown::new(Duration::from_micros(15)));
    cooldowns.global_cooldown = initial_gcd.clone();
    // Trigger the GCD
    let _ = cooldowns.trigger(&Action::Long);

    app.update();

    let cooldowns: &CooldownState<Action> = app.world().resource();
    assert_ne!(initial_gcd, cooldowns.global_cooldown);
}

#[test]
fn global_cooldown_blocks_cooldownless_actions() {
    let mut app = App::new();
    app.add_plugins(AbilityPlugin::<Action>::default())
        .add_plugins(MinimalPlugins)
        .add_plugins(InputPlugin)
        .insert_resource(Action::cooldowns());

    // First delta time provided of each app is wonky
    app.update();

    let mut cooldowns: Mut<CooldownState<Action>> = app.world_mut().resource_mut();
    cooldowns.global_cooldown = Some(Cooldown::new(Duration::from_micros(15)));

    assert!(cooldowns.ready(&Action::NoCooldown).is_ok());

    let _ = cooldowns.trigger(&Action::NoCooldown);
    assert_eq!(
        cooldowns.ready(&Action::NoCooldown),
        Err(CannotUseAbility::OnGlobalCooldown)
    );

    sleep(Duration::from_micros(30));
    app.update();

    let cooldowns: &CooldownState<Action> = app.world().resource();
    assert!(cooldowns.ready(&Action::NoCooldown).is_ok());
}

#[test]
fn global_cooldown_affects_other_actions() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        InputPlugin,
        AbilityPlugin::<Action>::default(),
    ))
    .insert_resource(Action::cooldowns());

    // First delta time provided of each app is wonky
    app.update();

    let mut cooldowns: Mut<CooldownState<Action>> = app.world_mut().resource_mut();
    cooldowns.global_cooldown = Some(Cooldown::new(Duration::from_micros(15)));
    let _ = cooldowns.trigger(&Action::Long);
    assert_eq!(
        cooldowns.ready(&Action::Short),
        Err(CannotUseAbility::OnGlobalCooldown)
    );
    assert_eq!(
        cooldowns.ready(&Action::Long),
        Err(CannotUseAbility::OnCooldown)
    );

    sleep(Duration::from_micros(30));
    app.update();

    let cooldowns: &CooldownState<Action> = app.world().resource();
    assert!(cooldowns.ready(&Action::Short).is_ok());
    assert_eq!(
        cooldowns.ready(&Action::Long),
        Err(CannotUseAbility::OnCooldown)
    );
}

#[test]
fn global_cooldown_overrides_short_cooldowns() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AbilityPlugin::<Action>::default(),
        InputPlugin,
    ))
    .insert_resource(Action::cooldowns());

    // First delta time provided of each app is wonky
    app.update();

    let mut cooldowns: Mut<CooldownState<Action>> = app.world_mut().resource_mut();
    cooldowns.global_cooldown = Some(Cooldown::from_secs(0.5));
    let _ = cooldowns.trigger(&Action::Short);
    assert_eq!(
        cooldowns.ready(&Action::Short),
        Err(CannotUseAbility::OnCooldown)
    );

    // Let per-action cooldown elapse
    sleep(Duration::from_millis(250));
    app.update();

    let cooldowns: &CooldownState<Action> = app.world().resource();
    assert_eq!(
        cooldowns.ready(&Action::Short),
        Err(CannotUseAbility::OnGlobalCooldown)
    );

    // Wait for full GCD to expire
    sleep(Duration::from_millis(250));
    app.update();

    let cooldowns: &CooldownState<Action> = app.world().resource();
    assert!(cooldowns.ready(&Action::Short).is_ok());
}

#[test]
fn cooldown_not_triggered_on_gcd() {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AbilityPlugin::<Action>::default(),
        InputPlugin,
    ))
    .insert_resource(Action::cooldowns());

    // First delta time provided of each app is wonky
    app.update();

    let mut cooldowns: Mut<CooldownState<Action>> = app.world_mut().resource_mut();
    cooldowns.global_cooldown = Some(Cooldown::from_secs(0.5));
    let _ = cooldowns.trigger(&Action::Long);
    assert_eq!(
        cooldowns.ready(&Action::Long),
        Err(CannotUseAbility::OnCooldown)
    );

    // Let per-action cooldown elapse
    sleep(Duration::from_millis(250));
    app.update();

    // Action::Short should be ready itself, but the GCD will prevent it
    // assert Action::Short is still ready after failing to trigger
    let mut cooldowns: Mut<CooldownState<Action>> = app.world_mut().resource_mut();
    assert_eq!(
        cooldowns.trigger(&Action::Short),
        Err(CannotUseAbility::OnGlobalCooldown)
    );

    let short_cooldown = cooldowns.get(&Action::Short).unwrap();
    assert!(short_cooldown.ready().is_ok());

    // Wait for full GCD to expire
    sleep(Duration::from_millis(250));
    app.update();

    let cooldowns: &CooldownState<Action> = app.world().resource();
    assert!(cooldowns.ready(&Action::Short).is_ok());
}
