use bevy::prelude::*;
use bevy::ui::UiScale;
use bevy::window::{CursorGrabMode, CursorOptions, WindowMode, WindowResolution};
use std::process;

const BASE_HEIGHT: f32 = 1080.0;

#[derive(Resource, Default)]
struct LastWindowHeight(f32);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_state::<MenuState>()
            .init_resource::<LastWindowHeight>()
            .add_systems(Startup, setup_menu)
            .add_systems(
                OnEnter(GameState::MainMenu),
                (show_main_menu, unlock_cursor),
            )
            .add_systems(OnExit(GameState::MainMenu), cleanup_menu)
            .add_systems(OnEnter(GameState::Paused), (show_pause_menu, unlock_cursor))
            .add_systems(OnExit(GameState::Paused), cleanup_menu)
            .add_systems(OnEnter(MenuState::Options), show_options_menu)
            .add_systems(OnExit(MenuState::Options), cleanup_options)
            .add_systems(OnEnter(GameState::Playing), lock_cursor)
            .add_systems(
                Update,
                (
                    handle_menu_buttons,
                    handle_options_buttons,
                    handle_pause_input,
                    update_ui_scale_on_change,
                    update_resolution_buttons_state,
                ),
            );
    }
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum MenuState {
    #[default]
    None,
    Options,
}

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct OptionsRoot;

#[derive(Component)]
enum MenuButton {
    Start,
    Resume,
    Options,
    Close,
}

#[derive(Component)]
enum OptionsButton {
    Fullscreen,
    Resolution(u32, u32),
    Back,
}

#[derive(Component)]
struct ResolutionButton;

#[derive(Component)]
struct ButtonText;

#[derive(Resource)]
struct MenuColors {
    normal: Color,
    hovered: Color,
    pressed: Color,
}

impl Default for MenuColors {
    fn default() -> Self {
        Self {
            normal: Color::srgb(0.15, 0.15, 0.15),
            hovered: Color::srgb(0.25, 0.25, 0.25),
            pressed: Color::srgb(0.35, 0.55, 0.35),
        }
    }
}

fn setup_menu(mut commands: Commands) {
    commands.insert_resource(MenuColors::default());
}

fn unlock_cursor(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.grab_mode = CursorGrabMode::None;
    cursor_options.visible = true;
}

fn lock_cursor(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}

fn show_main_menu(mut commands: Commands) {
    spawn_menu(
        &mut commands,
        "My Bevy Game",
        vec![
            ("Start", MenuButton::Start),
            ("Options", MenuButton::Options),
            ("Close", MenuButton::Close),
        ],
    );
}

fn show_pause_menu(mut commands: Commands) {
    spawn_menu(
        &mut commands,
        "Paused",
        vec![
            ("Resume", MenuButton::Resume),
            ("Options", MenuButton::Options),
            ("Close", MenuButton::Close),
        ],
    );
}

fn spawn_menu(commands: &mut Commands, title: &str, buttons: Vec<(&str, MenuButton)>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            MenuRoot,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Buttons
            for (text, button_type) in buttons {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(250.0),
                            height: Val::Px(65.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                        button_type,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Text::new(text),
                            TextFont {
                                font_size: 30.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }
        });
}

fn show_options_menu(mut commands: Commands, window: Single<&Window>) {
    let current_mode = &window.mode;
    let is_fullscreen = matches!(
        current_mode,
        WindowMode::Fullscreen(..) | WindowMode::BorderlessFullscreen(_)
    );

    let fullscreen_text = if is_fullscreen {
        "Fullscreen: ON"
    } else {
        "Fullscreen: OFF"
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            OptionsRoot,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Options"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Fullscreen toggle
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    OptionsButton::Fullscreen,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(fullscreen_text),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        ButtonText,
                    ));
                });

            // Resolution label
            parent.spawn((
                Text::new("Resolution:"),
                TextFont {
                    font_size: 25.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // Resolution buttons
            for (w, h, label) in [
                (1280u32, 720u32, "1280 x 720"),
                (1920, 1080, "1920 x 1080"),
                (2560, 1440, "2560 x 1440"),
            ] {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Px(300.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                        OptionsButton::Resolution(w, h),
                        ResolutionButton,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
            }

            // Back button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    OptionsButton::Back,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Back"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn cleanup_menu(mut commands: Commands, menu_query: Query<Entity, With<MenuRoot>>) {
    for entity in menu_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn cleanup_options(mut commands: Commands, options_query: Query<Entity, With<OptionsRoot>>) {
    for entity in options_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_menu_buttons(
    mut interaction_query: Query<
        (&Interaction, &MenuButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    colors: Res<MenuColors>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    for (interaction, button, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = colors.pressed.into();
                match button {
                    MenuButton::Start => {
                        next_game_state.set(GameState::Playing);
                    }
                    MenuButton::Resume => {
                        next_game_state.set(GameState::Playing);
                    }
                    MenuButton::Options => {
                        next_menu_state.set(MenuState::Options);
                    }
                    MenuButton::Close => {
                        // Use immediate exit to avoid slow cleanup with many physics entities
                        process::exit(0);
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = colors.hovered.into();
            }
            Interaction::None => {
                *bg_color = colors.normal.into();
            }
        }
    }
}

fn handle_options_buttons(
    mut interaction_query: Query<
        (
            &Interaction,
            &OptionsButton,
            &mut BackgroundColor,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text, With<ButtonText>>,
    colors: Res<MenuColors>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut window: Single<&mut Window>,
) {
    for (interaction, button, mut bg_color, children) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = colors.pressed.into();
                match button {
                    OptionsButton::Fullscreen => {
                        let is_fullscreen = matches!(
                            window.mode,
                            WindowMode::Fullscreen(..) | WindowMode::BorderlessFullscreen(_)
                        );

                        if is_fullscreen {
                            window.mode = WindowMode::Windowed;
                            // Reset to default resolution when exiting fullscreen
                            window.resolution = WindowResolution::new(1920, 1080);
                        } else {
                            window.mode =
                                WindowMode::BorderlessFullscreen(MonitorSelection::Current);
                        }

                        // Update button text
                        for child in children.iter() {
                            if let Ok(mut text) = text_query.get_mut(child) {
                                let new_text = if is_fullscreen {
                                    "Fullscreen: OFF"
                                } else {
                                    "Fullscreen: ON"
                                };
                                **text = new_text.to_string();
                            }
                        }
                    }
                    OptionsButton::Resolution(w, h) => {
                        // Only change resolution in windowed mode
                        if matches!(window.mode, WindowMode::Windowed) {
                            window.resolution = WindowResolution::new(*w, *h);
                        }
                    }
                    OptionsButton::Back => {
                        next_menu_state.set(MenuState::None);
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = colors.hovered.into();
            }
            Interaction::None => {
                *bg_color = colors.normal.into();
            }
        }
    }
}

fn handle_pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Playing => {
                next_state.set(GameState::Paused);
                next_menu_state.set(MenuState::None);
            }
            GameState::Paused => {
                next_state.set(GameState::Playing);
                next_menu_state.set(MenuState::None);
            }
            _ => {}
        }
    }
}

fn update_resolution_buttons_state(
    window: Single<&Window>,
    mut buttons: Query<(&mut BackgroundColor, &Children), With<ResolutionButton>>,
    mut text_query: Query<&mut TextColor>,
) {
    let is_fullscreen = matches!(
        window.mode,
        WindowMode::Fullscreen(..) | WindowMode::BorderlessFullscreen(_)
    );

    let (bg_color, text_color) = if is_fullscreen {
        // Grayed out in fullscreen
        (Color::srgb(0.1, 0.1, 0.1), Color::srgb(0.4, 0.4, 0.4))
    } else {
        // Normal in windowed
        (Color::srgb(0.15, 0.15, 0.15), Color::WHITE)
    };

    for (mut bg, children) in buttons.iter_mut() {
        *bg = bg_color.into();
        for child in children.iter() {
            if let Ok(mut tc) = text_query.get_mut(child) {
                tc.0 = text_color;
            }
        }
    }
}

fn update_ui_scale_on_change(
    window: Single<&Window>,
    mut ui_scale: ResMut<UiScale>,
    mut last_height: ResMut<LastWindowHeight>,
) {
    let current_height = window.height();

    // Only update if height actually changed
    if (current_height - last_height.0).abs() > 0.1 {
        last_height.0 = current_height;
        let scale = current_height / BASE_HEIGHT;
        ui_scale.0 = scale;
    }
}
