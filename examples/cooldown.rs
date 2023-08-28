//! Demonstrates how to store (and use) per-action cooldowns
//!
//! This example shows off a tiny cookie clicker!
use bevy::{prelude::*, reflect::Reflect};
use leafwing_abilities::prelude::*;
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::*};

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

#[derive(Actionlike, Reflect, Abilitylike, Clone, Copy, PartialEq, Debug, Default)]
enum CookieAbility {
    #[default]
    AddOne,
    DoubleCookies,
}

impl CookieAbility {
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
        InputMap::default()
            .insert(KeyCode::Space, CookieAbility::DoubleCookies)
            .build()
    }
}

/// Marker component for our clickable cookies
#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct Cookie;

#[derive(Bundle)]
struct CookieBundle {
    cookie: Cookie,
    button_bundle: ButtonBundle,
    abilities_bundle: AbilitiesBundle<CookieAbility>,
    input_manager_bundle: InputManagerBundle<CookieAbility>,
}

impl CookieBundle {
    const COOKIE_CLICKED_COLOR: Color = Color::BEIGE;
    const COOKIE_COLOR: Color = Color::GOLD;

    /// Creates a Cookie bundle with a random position.
    fn new() -> CookieBundle {
        CookieBundle {
            cookie: Cookie,
            button_bundle: ButtonBundle {
                style: Style {
                    height: Val::Px(100.),
                    width: Val::Px(100.),
                    ..Default::default()
                },
                background_color: BackgroundColor(Self::COOKIE_COLOR),
                ..default()
            },
            abilities_bundle: AbilitiesBundle {
                cooldowns: CookieAbility::cooldowns(),
                ..default()
            },
            input_manager_bundle: InputManagerBundle {
                action_state: Default::default(),
                input_map: CookieAbility::key_bindings(),
            },
        }
    }
}

fn spawn_cookie(mut commands: Commands) {
    commands.spawn(CookieBundle::new());
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// We need a huge amount of space to be able to let you play this game for long enough ;)
#[derive(Resource, Default)]
struct Score(u128);

fn cookie_clicked(mut query: Query<(&Interaction, &mut ActionState<CookieAbility>)>) {
    let (cookie_interaction, mut cookie_action_state) = query.single_mut();
    // This indirection is silly here, but works well in larger games
    // by allowing you to hook into the ability state.
    if *cookie_interaction == Interaction::Pressed {
        cookie_action_state.press(CookieAbility::AddOne);
    }
}

fn handle_add_one_ability(
    mut query: Query<(
        &ActionState<CookieAbility>,
        &mut CooldownState<CookieAbility>,
    )>,
    mut score: ResMut<Score>,
) {
    let (actions, mut cooldowns) = query.single_mut();
    // See the handle_double_cookies system for a more ergonomic, robust (and implicit) way to handle this pattern
    if actions.just_pressed(CookieAbility::AddOne) {
        // Calling .trigger checks if the cooldown can be used, then triggers it if so
        // Note that this may miss other important limitations on when abilities can be used
        if cooldowns.trigger(CookieAbility::AddOne).is_ok() {
            // The result returned should be checked to decide how to respond
            score.0 += 1;
        }
    }
}

fn handle_double_cookies_ability(
    mut query: Query<AbilityState<CookieAbility>>,
    mut score: ResMut<Score>,
) {
    let mut cookie_ability_state = query.single_mut();
    // Checks whether the action is pressed, and if it is ready.
    // If so, triggers the ability, resetting its cooldown.
    if cookie_ability_state
        .trigger_if_just_pressed(CookieAbility::DoubleCookies)
        .is_ok()
    {
        score.0 *= 2;
    }
}

fn change_cookie_color_when_clicked(
    mut query: Query<(&mut BackgroundColor, AbilityState<CookieAbility>)>,
) {
    let (mut color, ability_state) = query.single_mut();
    if ability_state
        .ready_and_just_pressed(CookieAbility::AddOne)
        .is_ok()
    {
        *color = CookieBundle::COOKIE_CLICKED_COLOR.into();
    }
}

/// Resets the cookie's color after a frame
fn reset_cookie_color(mut query: Query<&mut BackgroundColor, With<Cookie>>) {
    let mut color = query.single_mut();
    *color = CookieBundle::COOKIE_COLOR.into();
}

#[derive(Component)]
struct ScoreText;

fn spawn_score_text(mut commands: Commands) {
    commands
        .spawn(TextBundle::from_section(
            "Score: ",
            TextStyle {
                font_size: 50.,
                color: Color::WHITE,
                ..Default::default()
            },
        ))
        .insert(ScoreText);
}

fn display_score(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    let score = score.0;
    let mut text = query.single_mut();
    text.sections[0].value = format!("Score: {score}");
}
