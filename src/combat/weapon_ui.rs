use super::{ReloadState, WeaponInventory};
use crate::player::{Player, PlayerHealth};
use crate::ui::GameState;
use bevy::prelude::*;

pub struct WeaponUiPlugin;

impl Plugin for WeaponUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (spawn_weapon_hud, spawn_health_hud),
        )
        .add_systems(
            OnExit(GameState::Playing),
            (despawn_weapon_hud, despawn_health_hud),
        )
        .add_systems(
            Update,
            (update_weapon_hud, update_health_hud).run_if(in_state(GameState::Playing)),
        );
    }
}

// === WEAPON HUD (bottom-right) ===

#[derive(Component)]
struct WeaponHud;

#[derive(Component)]
struct WeaponNameText;

#[derive(Component)]
struct FireModeText;

#[derive(Component)]
struct AmmoText;

#[derive(Component)]
struct ReloadIndicator;

fn spawn_weapon_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                bottom: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                padding: UiRect::all(Val::Px(15.0)),
                row_gap: Val::Px(5.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            WeaponHud,
        ))
        .with_children(|parent| {
            // Weapon name
            parent.spawn((
                Text::new("PISTOL"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                WeaponNameText,
            ));

            // Fire mode
            parent.spawn((
                Text::new("Semi-Auto"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                FireModeText,
            ));

            // Ammo count
            parent.spawn((
                Text::new("12 / 48"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.9, 0.3)),
                AmmoText,
            ));

            // Reload indicator (hidden by default)
            parent.spawn((
                Text::new("RELOADING..."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
                Visibility::Hidden,
                ReloadIndicator,
            ));
        });
}

fn despawn_weapon_hud(mut commands: Commands, hud_query: Query<Entity, With<WeaponHud>>) {
    for entity in hud_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn update_weapon_hud(
    player_query: Query<(&WeaponInventory, Option<&ReloadState>), With<Player>>,
    mut weapon_name_query: Query<
        &mut Text,
        (
            With<WeaponNameText>,
            Without<FireModeText>,
            Without<AmmoText>,
        ),
    >,
    mut fire_mode_query: Query<
        &mut Text,
        (
            With<FireModeText>,
            Without<WeaponNameText>,
            Without<AmmoText>,
        ),
    >,
    mut ammo_query: Query<
        &mut Text,
        (
            With<AmmoText>,
            Without<WeaponNameText>,
            Without<FireModeText>,
        ),
    >,
    mut reload_query: Query<&mut Visibility, With<ReloadIndicator>>,
) {
    let Ok((inventory, reload_state)) = player_query.single() else {
        return;
    };

    let Some(weapon) = inventory.current_weapon() else {
        return;
    };

    // Update weapon name
    for mut text in weapon_name_query.iter_mut() {
        **text = weapon.weapon_type.name().to_string();
    }

    // Update fire mode
    for mut text in fire_mode_query.iter_mut() {
        **text = weapon.fire_mode.name().to_string();
    }

    // Update ammo
    for mut text in ammo_query.iter_mut() {
        **text = format!("{} / {}", weapon.current_ammo, weapon.reserve_ammo);
    }

    // Update reload indicator visibility
    for mut visibility in reload_query.iter_mut() {
        *visibility = if reload_state.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

// === HEALTH HUD (top-left) ===

#[derive(Component)]
struct HealthHud;

#[derive(Component)]
struct HealthText;

#[derive(Component)]
struct HealthBarContainer;

#[derive(Component)]
struct HealthBarFill;

fn spawn_health_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                top: Val::Px(20.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(15.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            HealthHud,
        ))
        .with_children(|parent| {
            // Health label
            parent.spawn((
                Text::new("HEALTH"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // Health bar container
            parent
                .spawn((
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    HealthBarContainer,
                ))
                .with_children(|bar_parent| {
                    // Health bar fill
                    bar_parent.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
                        HealthBarFill,
                    ));
                });

            // Health text (number)
            parent.spawn((
                Text::new("100 / 100"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                HealthText,
            ));
        });
}

fn despawn_health_hud(mut commands: Commands, hud_query: Query<Entity, With<HealthHud>>) {
    for entity in hud_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn update_health_hud(
    player_query: Query<&PlayerHealth, With<Player>>,
    mut health_text_query: Query<&mut Text, With<HealthText>>,
    mut health_bar_query: Query<(&mut Node, &mut BackgroundColor), With<HealthBarFill>>,
) {
    let Ok(health) = player_query.single() else {
        return;
    };

    // Update health text
    for mut text in health_text_query.iter_mut() {
        **text = format!("{:.0} / {:.0}", health.current, health.max);
    }

    // Update health bar width and color
    for (mut node, mut bg_color) in health_bar_query.iter_mut() {
        let health_percent = (health.current / health.max).clamp(0.0, 1.0);
        node.width = Val::Percent(health_percent * 100.0);

        // Change color based on health level
        let color = if health_percent > 0.5 {
            Color::srgb(0.2, 0.8, 0.2) // Green
        } else if health_percent > 0.25 {
            Color::srgb(0.8, 0.8, 0.2) // Yellow
        } else {
            Color::srgb(0.8, 0.2, 0.2) // Red
        };
        *bg_color = BackgroundColor(color);
    }
}
