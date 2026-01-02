use crate::menu::GameState;
use crate::player::Player;
use crate::shooting::{ReloadState, WeaponInventory};
use bevy::prelude::*;

pub struct WeaponUiPlugin;

impl Plugin for WeaponUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_weapon_hud)
            .add_systems(OnExit(GameState::Playing), despawn_weapon_hud)
            .add_systems(
                Update,
                update_weapon_hud.run_if(in_state(GameState::Playing)),
            );
    }
}

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
