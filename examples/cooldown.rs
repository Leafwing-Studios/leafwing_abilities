//! Demonstrates how to store (and use) per-action cooldowns
//!
//! This example shows off a tiny cookie clicker!
use bevy::{prelude::*, reflect::Reflect};
use leafwing_abilities::prelude::*;
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::*};

use bevy::color::palettes::css::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(InputManagerPlugin::<CookieAbility>::default())
        .add_plugins(AbilityPlugin::<CookieAbility>::default())
        .add_systems(Startup, (spawn_cookie, spawn_camera, spawn_score_text))
        .init_resource::<Score>()
        // We're manually calling ActionState::press, so we have to get the timing right so just_pressed isn't overridden
        .add_systems(PreUpdate, cookie_clicked.after(InputManagerSystem::Update))
        .add_systems(
            Update,
            (
                handle_add_one_ability,
                handle_double_cookies_ability,
                change_cookie_color_when_clicked.before(handle_add_one_ability),
            ),
        )
        // Reset the cookie's color when clicked after a single frame
        // Rendering happens after CoreStage::Update, so this should do the trick
        .add_systems(PreUpdate, reset_cookie_color)
        // Only the freshest scores here
        .add_systems(PostUpdate, display_score)
        .run();
}

#[derive(Actionlike, Reflect, Abilitylike, Clone, Copy, PartialEq, Debug, Default, Hash, Eq)]
enum CookieAbility {
    #[default]
    AddOne,
    DoubleCookies,
}

impl CookieAbility {
    /// You could use the `strum` crate to derive this automatically!
    fn variants() -> impl Iterator<Item = CookieAbility> {
        use CookieAbility::*;
        [AddOne, DoubleCookies].iter().copied()
    }

    fn cooldown(&self) -> Cooldown {
        match self {
            CookieAbility::AddOne => Cooldown::from_secs(0.1),
            CookieAbility::DoubleCookies => Cooldown::from_secs(5.0),
        }
    }

    fn cooldowns() -> CooldownState<CookieAbility> {
        let mut cooldowns = CooldownState::default();
        for ability in CookieAbility::variants() {
            cooldowns.set(ability, ability.cooldown());
        }
        cooldowns
    }

    fn key_bindings() -> InputMap<CookieAbility> {
        // CookieAbility::AddOne is pressed manually when the cookie is clicked on
        InputMap::default().with(CookieAbility::DoubleCookies, KeyCode::Space)
    }
}

/// Marker component for our clickable cookies
#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct Cookie;

#[derive(Bundle)]
struct CookieBundle {
    cookie: Cookie,
    node: Node,
    background_color: BackgroundColor,
    abilities_bundle: AbilitiesBundle<CookieAbility>,
    input_map: InputMap<CookieAbility>,
}

impl CookieBundle {
    const COOKIE_CLICKED_COLOR: Srgba = BEIGE;
    const COOKIE_COLOR: Srgba = BROWN;

    /// Creates a Cookie bundle with a random position.
    fn new() -> CookieBundle {
        CookieBundle {
            cookie: Cookie,
            node: Node {
                height: Val::Px(100.),
                width: Val::Px(100.),
                ..Default::default()
            },
            background_color: BackgroundColor(Self::COOKIE_COLOR.into()),
            abilities_bundle: AbilitiesBundle {
                cooldowns: CookieAbility::cooldowns(),
                ..default()
            },
            input_map: CookieAbility::key_bindings(),
        }
    }
}

fn spawn_cookie(mut commands: Commands) {
    commands.spawn(CookieBundle::new());
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// We need a huge amount of space to be able to let you play this game for long enough ;)
#[derive(Resource, Default)]
struct Score(u128);

fn cookie_clicked(mut query: Query<(&Interaction, &mut ActionState<CookieAbility>)>) -> Result {
    let (cookie_interaction, mut cookie_action_state) = query.single_mut()?;
    // This indirection is silly here, but works well in larger games
    // by allowing you to hook into the ability state.
    if *cookie_interaction == Interaction::Pressed {
        cookie_action_state.press(&CookieAbility::AddOne);
    }

    Ok(())
}

fn handle_add_one_ability(
    mut query: Query<(
        &ActionState<CookieAbility>,
        &mut CooldownState<CookieAbility>,
    )>,
    mut score: ResMut<Score>,
) -> Result {
    let (actions, mut cooldowns) = query.single_mut()?;
    // See the handle_double_cookies system for a more ergonomic, robust (and implicit) way to handle this pattern
    if actions.just_pressed(&CookieAbility::AddOne) {
        // Calling .trigger checks if the cooldown can be used, then triggers it if so
        // Note that this may miss other important limitations on when abilities can be used
        if cooldowns.trigger(&CookieAbility::AddOne).is_ok() {
            // The result returned should be checked to decide how to respond
            score.0 += 1;
        }
    }

    Ok(())
}

fn handle_double_cookies_ability(
    mut query: Query<AbilityState<CookieAbility>>,
    mut score: ResMut<Score>,
) -> Result {
    let mut cookie_ability_state = query.single_mut()?;
    // Checks whether the action is pressed, and if it is ready.
    // If so, triggers the ability, resetting its cooldown.
    if cookie_ability_state
        .trigger_if_just_pressed(&CookieAbility::DoubleCookies)
        .is_ok()
    {
        score.0 *= 2;
    }

    Ok(())
}

fn change_cookie_color_when_clicked(
    mut query: Query<(&mut BackgroundColor, AbilityState<CookieAbility>)>,
) -> Result {
    let (mut color, ability_state) = query.single_mut()?;
    if ability_state
        .ready_and_just_pressed(&CookieAbility::AddOne)
        .is_ok()
    {
        *color = CookieBundle::COOKIE_CLICKED_COLOR.into();
    }

    Ok(())
}

/// Resets the cookie's color after a frame
fn reset_cookie_color(mut query: Query<&mut BackgroundColor, With<Cookie>>) -> Result {
    let mut color = query.single_mut()?;
    *color = CookieBundle::COOKIE_COLOR.into();
    Ok(())
}

#[derive(Component)]
struct ScoreText;

fn spawn_score_text(mut commands: Commands) {
    commands.spawn(Text::new("Score")).insert(ScoreText);
}

fn display_score(score: Res<Score>, mut text: Single<&mut Text, With<ScoreText>>) {
    let score = score.0;
    **text = Text::new(format!("Score: {}", score));
}
