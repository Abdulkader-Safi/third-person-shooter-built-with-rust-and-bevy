use crate::menu::GameState;
use crate::player::Player;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

pub struct ShootingPlugin;

impl Plugin for ShootingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HitEvent>().add_systems(
            Update,
            (
                handle_weapon_switch,
                handle_reload_input,
                process_reload,
                process_burst,
                shoot,
                update_shoot_cooldown,
                update_debug_rays,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// =============================================================================
// WEAPON TYPES AND DATA
// =============================================================================

/// Types of weapons available in the game
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum WeaponType {
    #[default]
    Pistol,
    Smg,
    Rifle,
    Shotgun,
}

impl WeaponType {
    pub fn name(&self) -> &'static str {
        match self {
            WeaponType::Pistol => "PISTOL",
            WeaponType::Smg => "SMG",
            WeaponType::Rifle => "RIFLE",
            WeaponType::Shotgun => "SHOTGUN",
        }
    }
}

/// Fire modes for weapons
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FireMode {
    #[default]
    SemiAuto,
    FullAuto,
    Burst(u8), // Number of shots per burst
}

impl FireMode {
    pub fn name(&self) -> &'static str {
        match self {
            FireMode::SemiAuto => "Semi-Auto",
            FireMode::FullAuto => "Full-Auto",
            FireMode::Burst(_) => "Burst",
        }
    }
}

/// Weapon component with all stats
#[derive(Component, Clone, Debug)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub fire_mode: FireMode,
    pub damage: f32,
    pub fire_rate: f32, // Shots per second
    pub pellets: u8,    // 1 for most, 6 for shotgun
    pub spread: f32,    // Spread angle in radians
    pub magazine_size: u32,
    pub current_ammo: u32,
    pub reserve_ammo: u32,
    pub reload_time: f32, // Seconds
}

impl Default for Weapon {
    fn default() -> Self {
        Self::pistol()
    }
}

impl Weapon {
    /// Create a pistol - Semi-auto, reliable damage
    pub fn pistol() -> Self {
        Self {
            weapon_type: WeaponType::Pistol,
            fire_mode: FireMode::SemiAuto,
            damage: 15.0,
            fire_rate: 3.0,
            pellets: 1,
            spread: 0.0,
            magazine_size: 12,
            current_ammo: 12,
            reserve_ammo: 48,
            reload_time: 1.5,
        }
    }

    /// Create an SMG - Full-auto, high fire rate, low damage
    pub fn smg() -> Self {
        Self {
            weapon_type: WeaponType::Smg,
            fire_mode: FireMode::FullAuto,
            damage: 8.0,
            fire_rate: 10.0,
            pellets: 1,
            spread: 0.02,
            magazine_size: 30,
            current_ammo: 30,
            reserve_ammo: 120,
            reload_time: 2.0,
        }
    }

    /// Create a rifle - 3-round burst, high damage
    pub fn rifle() -> Self {
        Self {
            weapon_type: WeaponType::Rifle,
            fire_mode: FireMode::Burst(3),
            damage: 35.0,
            fire_rate: 2.0,
            pellets: 1,
            spread: 0.01,
            magazine_size: 20,
            current_ammo: 20,
            reserve_ammo: 60,
            reload_time: 2.5,
        }
    }

    /// Create a shotgun - Semi-auto, 6 pellets with spread
    pub fn shotgun() -> Self {
        Self {
            weapon_type: WeaponType::Shotgun,
            fire_mode: FireMode::SemiAuto,
            damage: 12.0, // Per pellet
            fire_rate: 1.0,
            pellets: 6,
            spread: 0.15, // Wide spread
            magazine_size: 6,
            current_ammo: 6,
            reserve_ammo: 24,
            reload_time: 2.5,
        }
    }

    /// Check if magazine is empty
    pub fn is_empty(&self) -> bool {
        self.current_ammo == 0
    }

    /// Check if can reload (has reserve ammo and not full)
    pub fn can_reload(&self) -> bool {
        self.reserve_ammo > 0 && self.current_ammo < self.magazine_size
    }

    /// Get cooldown duration between shots
    pub fn shot_cooldown(&self) -> f32 {
        1.0 / self.fire_rate
    }
}

// =============================================================================
// INVENTORY AND STATE COMPONENTS
// =============================================================================

/// Holds all weapons the player has
#[derive(Component)]
pub struct WeaponInventory {
    pub weapons: [Option<Weapon>; 4],
    pub current_slot: usize,
}

impl Default for WeaponInventory {
    fn default() -> Self {
        Self {
            weapons: [
                Some(Weapon::pistol()),
                Some(Weapon::smg()),
                Some(Weapon::rifle()),
                Some(Weapon::shotgun()),
            ],
            current_slot: 0,
        }
    }
}

impl WeaponInventory {
    /// Get the currently equipped weapon
    pub fn current_weapon(&self) -> Option<&Weapon> {
        self.weapons[self.current_slot].as_ref()
    }

    /// Get mutable reference to current weapon
    pub fn current_weapon_mut(&mut self) -> Option<&mut Weapon> {
        self.weapons[self.current_slot].as_mut()
    }

    /// Switch to a specific slot (0-3)
    pub fn switch_to(&mut self, slot: usize) {
        if slot < 4 && self.weapons[slot].is_some() {
            self.current_slot = slot;
        }
    }

    /// Cycle to next available weapon
    pub fn cycle_next(&mut self) {
        for i in 1..=4 {
            let next_slot = (self.current_slot + i) % 4;
            if self.weapons[next_slot].is_some() {
                self.current_slot = next_slot;
                return;
            }
        }
    }

    /// Cycle to previous available weapon
    pub fn cycle_prev(&mut self) {
        for i in 1..=4 {
            let prev_slot = (self.current_slot + 4 - i) % 4;
            if self.weapons[prev_slot].is_some() {
                self.current_slot = prev_slot;
                return;
            }
        }
    }
}

/// Cooldown timer between shots
#[derive(Component)]
pub struct ShootCooldown(pub Timer);

impl Default for ShootCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(0.0, TimerMode::Once))
    }
}

/// Active reload state
#[derive(Component)]
pub struct ReloadState(pub Timer);

/// Burst fire state - tracks remaining shots in burst
#[derive(Component)]
pub struct BurstState {
    pub shots_remaining: u8,
    pub timer: Timer,
}

// =============================================================================
// MARKERS AND EVENTS
// =============================================================================

/// Add this component to any entity that can be shot
#[derive(Component)]
pub struct Shootable;

/// Event sent when something is shot
#[derive(Message)]
pub struct HitEvent {
    pub entity: Entity,
    pub damage: f32,
}

/// Debug ray visualization
#[derive(Component)]
pub struct DebugRay {
    pub timer: Timer,
}

// =============================================================================
// SYSTEMS
// =============================================================================

fn handle_weapon_switch(
    keys: Res<ButtonInput<KeyCode>>,
    scroll: Res<AccumulatedMouseScroll>,
    mut players: Query<(&mut WeaponInventory, Option<&ReloadState>), With<Player>>,
) {
    let Ok((mut inventory, reload_state)) = players.single_mut() else {
        return;
    };

    // Can't switch weapons while reloading
    if reload_state.is_some() {
        return;
    }

    // Number keys 1-4
    if keys.just_pressed(KeyCode::Digit1) {
        inventory.switch_to(0);
    } else if keys.just_pressed(KeyCode::Digit2) {
        inventory.switch_to(1);
    } else if keys.just_pressed(KeyCode::Digit3) {
        inventory.switch_to(2);
    } else if keys.just_pressed(KeyCode::Digit4) {
        inventory.switch_to(3);
    }

    // Scroll wheel
    let scroll_y = scroll.delta.y;
    if scroll_y > 0.0 {
        inventory.cycle_prev();
    } else if scroll_y < 0.0 {
        inventory.cycle_next();
    }
}

fn handle_reload_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    players: Query<(Entity, &WeaponInventory, Option<&ReloadState>), With<Player>>,
) {
    let Ok((entity, inventory, reload_state)) = players.single() else {
        return;
    };

    // Already reloading
    if reload_state.is_some() {
        return;
    }

    let should_reload = keys.just_pressed(KeyCode::KeyR);
    let auto_reload = inventory
        .current_weapon()
        .map(|w| w.is_empty())
        .unwrap_or(false);

    if should_reload || auto_reload {
        if let Some(weapon) = inventory.current_weapon() {
            if weapon.can_reload() {
                let reload_time = weapon.reload_time;
                commands
                    .entity(entity)
                    .insert(ReloadState(Timer::from_seconds(
                        reload_time,
                        TimerMode::Once,
                    )));
            }
        }
    }
}

fn process_reload(
    mut commands: Commands,
    time: Res<Time>,
    mut players: Query<(Entity, &mut WeaponInventory, &mut ReloadState), With<Player>>,
) {
    for (entity, mut inventory, mut reload) in players.iter_mut() {
        reload.0.tick(time.delta());

        if reload.0.is_finished() {
            if let Some(weapon) = inventory.current_weapon_mut() {
                let needed = weapon.magazine_size - weapon.current_ammo;
                let available = weapon.reserve_ammo.min(needed);
                weapon.current_ammo += available;
                weapon.reserve_ammo -= available;
            }
            commands.entity(entity).remove::<ReloadState>();
        }
    }
}

fn process_burst(
    mut commands: Commands,
    time: Res<Time>,
    mut players: Query<
        (
            Entity,
            &Transform,
            &mut WeaponInventory,
            &mut BurstState,
        ),
        With<Player>,
    >,
    rapier_context: ReadRapierContext,
    shootables: Query<Entity, With<Shootable>>,
    mut hit_events: MessageWriter<HitEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(context) = rapier_context.single() else {
        return;
    };

    for (player_entity, player_transform, mut inventory, mut burst) in players.iter_mut() {
        burst.timer.tick(time.delta());

        if burst.timer.is_finished() && burst.shots_remaining > 0 {
            if let Some(weapon) = inventory.current_weapon_mut() {
                if weapon.current_ammo > 0 {
                    // Fire one shot of the burst
                    fire_weapon(
                        &mut commands,
                        player_entity,
                        player_transform,
                        weapon,
                        &context,
                        &shootables,
                        &mut hit_events,
                        &mut meshes,
                        &mut materials,
                    );

                    burst.shots_remaining -= 1;
                    burst.timer.reset();
                } else {
                    burst.shots_remaining = 0;
                }
            }
        }

        if burst.shots_remaining == 0 {
            commands.entity(player_entity).remove::<BurstState>();
        }
    }
}

fn shoot(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut players: Query<
        (
            Entity,
            &Transform,
            &mut WeaponInventory,
            &mut ShootCooldown,
            Option<&ReloadState>,
            Option<&BurstState>,
        ),
        With<Player>,
    >,
    rapier_context: ReadRapierContext,
    shootables: Query<Entity, With<Shootable>>,
    mut hit_events: MessageWriter<HitEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(context) = rapier_context.single() else {
        return;
    };

    for (player_entity, player_transform, mut inventory, mut cooldown, reload_state, burst_state) in
        players.iter_mut()
    {
        // Can't shoot while reloading or in burst
        if reload_state.is_some() || burst_state.is_some() {
            continue;
        }

        // Check cooldown
        if !cooldown.0.is_finished() {
            continue;
        }

        let Some(weapon) = inventory.current_weapon() else {
            continue;
        };

        // Check ammo
        if weapon.is_empty() {
            continue;
        }

        // Check fire mode input
        let should_fire = match weapon.fire_mode {
            FireMode::SemiAuto => mouse_button.just_pressed(MouseButton::Left),
            FireMode::FullAuto => mouse_button.pressed(MouseButton::Left),
            FireMode::Burst(_) => mouse_button.just_pressed(MouseButton::Left),
        };

        if !should_fire {
            continue;
        }

        // Handle burst mode specially
        if let FireMode::Burst(count) = weapon.fire_mode {
            let cooldown_time = weapon.shot_cooldown();
            commands.entity(player_entity).insert(BurstState {
                shots_remaining: count,
                timer: Timer::from_seconds(0.0, TimerMode::Repeating),
            });
            cooldown.0 = Timer::from_seconds(cooldown_time, TimerMode::Once);
            continue;
        }

        // Fire the weapon
        let weapon_mut = inventory.current_weapon_mut().unwrap();
        fire_weapon(
            &mut commands,
            player_entity,
            player_transform,
            weapon_mut,
            &context,
            &shootables,
            &mut hit_events,
            &mut meshes,
            &mut materials,
        );

        // Set cooldown
        let cooldown_time = weapon_mut.shot_cooldown();
        cooldown.0 = Timer::from_seconds(cooldown_time, TimerMode::Once);
    }
}

fn fire_weapon(
    commands: &mut Commands,
    player_entity: Entity,
    player_transform: &Transform,
    weapon: &mut Weapon,
    context: &RapierContext,
    shootables: &Query<Entity, With<Shootable>>,
    hit_events: &mut MessageWriter<HitEvent>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    weapon.current_ammo -= 1;

    let player_pos = player_transform.translation;
    let player_forward = *player_transform.forward();
    let ray_origin = player_pos + Vec3::Y * 0.5;
    let max_distance = 100.0;

    let filter = QueryFilter::default().exclude_rigid_body(player_entity);

    // Generate ray directions based on pellet count and spread
    let directions = generate_spread_directions(player_forward, weapon.spread, weapon.pellets);

    for ray_direction in directions {
        let mut hit_entity: Option<(Entity, f32)> = None;
        context.with_query_pipeline(filter, |query_pipeline| {
            hit_entity = query_pipeline.cast_ray(ray_origin, ray_direction, max_distance, true);
        });

        // Debug ray visualization
        let ray_end = if let Some((_, distance)) = hit_entity {
            ray_origin + ray_direction * distance
        } else {
            ray_origin + ray_direction * max_distance
        };

        spawn_debug_ray(
            commands,
            ray_origin,
            ray_end,
            ray_direction,
            meshes,
            materials,
        );

        // Send hit event
        if let Some((entity, _)) = hit_entity {
            if shootables.get(entity).is_ok() {
                hit_events.write(HitEvent {
                    entity,
                    damage: weapon.damage,
                });
            }
        }
    }
}

fn generate_spread_directions(forward: Vec3, spread: f32, pellet_count: u8) -> Vec<Vec3> {
    if pellet_count == 1 && spread == 0.0 {
        return vec![forward];
    }

    let mut rng = rand::rng();
    let mut directions = Vec::with_capacity(pellet_count as usize);

    // Find perpendicular vectors for spread calculation
    let right = forward.cross(Vec3::Y).normalize_or_zero();
    let up = right.cross(forward).normalize_or_zero();

    for _ in 0..pellet_count {
        let angle = rng.random::<f32>() * std::f32::consts::TAU;
        let radius = rng.random::<f32>().sqrt() * spread;

        let offset_x = radius * angle.cos();
        let offset_y = radius * angle.sin();

        let direction = (forward + right * offset_x + up * offset_y).normalize();
        directions.push(direction);
    }

    directions
}

fn spawn_debug_ray(
    commands: &mut Commands,
    ray_origin: Vec3,
    ray_end: Vec3,
    ray_direction: Vec3,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let ray_length = (ray_end - ray_origin).length();
    let ray_center = (ray_origin + ray_end) / 2.0;
    let ray_rotation = Quat::from_rotation_arc(Vec3::Y, ray_direction);

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.02, ray_length, 0.02))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 1.0, 0.0),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(ray_center).with_rotation(ray_rotation),
        DebugRay {
            timer: Timer::from_seconds(1.5, TimerMode::Once),
        },
    ));
}

fn update_shoot_cooldown(time: Res<Time>, mut cooldowns: Query<&mut ShootCooldown>) {
    for mut cooldown in cooldowns.iter_mut() {
        cooldown.0.tick(time.delta());
    }
}

fn update_debug_rays(
    mut commands: Commands,
    time: Res<Time>,
    mut rays: Query<(Entity, &mut DebugRay)>,
) {
    for (entity, mut ray) in rays.iter_mut() {
        ray.timer.tick(time.delta());
        if ray.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
