use crate::nav_grid::NavGrid;
use crate::shooting::Shootable;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use rand::Rng;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_light, spawn_floor, spawn_obstacles).chain());
    }
}

/// Obstacle component - marks entities as obstacles
#[derive(Component)]
pub struct Obstacle {
    pub destructible: bool,
    pub health: f32,
    pub max_health: f32,
}

impl Obstacle {
    pub fn destructible(health: f32) -> Self {
        Self {
            destructible: true,
            health,
            max_health: health,
        }
    }

    pub fn indestructible() -> Self {
        Self {
            destructible: false,
            health: f32::MAX,
            max_health: f32::MAX,
        }
    }
}

fn spawn_light(mut commands: Commands) {
    // Main directional light (sun-like)
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 50.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light for better visibility
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 500.0,
        ..default()
    });
}

fn spawn_floor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 100x100 ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(100.0, 100.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.15, 0.35, 0.15))),
        RigidBody::Fixed,
        Collider::cuboid(50.0, 0.01, 50.0),
    ));
}

fn spawn_obstacles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut nav_grid: ResMut<NavGrid>,
) {
    let mut rng = rand::rng();

    // Materials
    let wall_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.4, 0.45),
        ..default()
    });
    let crate_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.55, 0.35, 0.15),
        ..default()
    });
    let barrel_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.2, 0.15),
        ..default()
    });
    let pillar_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        ..default()
    });

    // Meshes
    let wall_mesh = meshes.add(Cuboid::new(10.0, 3.0, 0.5));
    let crate_mesh = meshes.add(Cuboid::new(1.5, 1.5, 1.5));
    let barrel_mesh = meshes.add(Cylinder::new(0.5, 1.5));
    let pillar_mesh = meshes.add(Cuboid::new(1.0, 4.0, 1.0));

    // === PERIMETER WALLS ===
    // North wall
    spawn_wall(
        &mut commands,
        &wall_mesh,
        &wall_material,
        Vec3::new(0.0, 1.5, -49.0),
        100.0,
        3.0,
        0.5,
        0.0,
        &mut nav_grid,
    );
    // South wall
    spawn_wall(
        &mut commands,
        &wall_mesh,
        &wall_material,
        Vec3::new(0.0, 1.5, 49.0),
        100.0,
        3.0,
        0.5,
        0.0,
        &mut nav_grid,
    );
    // East wall
    spawn_wall(
        &mut commands,
        &wall_mesh,
        &wall_material,
        Vec3::new(49.0, 1.5, 0.0),
        0.5,
        3.0,
        100.0,
        std::f32::consts::FRAC_PI_2,
        &mut nav_grid,
    );
    // West wall
    spawn_wall(
        &mut commands,
        &wall_mesh,
        &wall_material,
        Vec3::new(-49.0, 1.5, 0.0),
        0.5,
        3.0,
        100.0,
        std::f32::consts::FRAC_PI_2,
        &mut nav_grid,
    );

    // === INTERNAL WALLS (10-15 segments) ===
    let wall_positions = [
        (Vec3::new(-25.0, 1.5, -20.0), 0.0),
        (Vec3::new(20.0, 1.5, -15.0), std::f32::consts::FRAC_PI_4),
        (Vec3::new(-15.0, 1.5, 25.0), 0.0),
        (Vec3::new(30.0, 1.5, 20.0), std::f32::consts::FRAC_PI_2),
        (Vec3::new(0.0, 1.5, -35.0), std::f32::consts::FRAC_PI_4),
        (Vec3::new(-35.0, 1.5, 0.0), 0.0),
        (Vec3::new(35.0, 1.5, -5.0), std::f32::consts::FRAC_PI_2),
        (Vec3::new(-10.0, 1.5, -10.0), std::f32::consts::FRAC_PI_4),
        (Vec3::new(15.0, 1.5, 30.0), 0.0),
        (Vec3::new(-30.0, 1.5, -35.0), std::f32::consts::FRAC_PI_2),
        (Vec3::new(25.0, 1.5, -30.0), 0.0),
        (Vec3::new(-20.0, 1.5, 15.0), std::f32::consts::FRAC_PI_4),
    ];

    for (pos, rotation) in wall_positions {
        commands.spawn((
            Mesh3d(wall_mesh.clone()),
            MeshMaterial3d(wall_material.clone()),
            Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(rotation)),
            Obstacle::indestructible(),
            RigidBody::Fixed,
            Collider::cuboid(5.0, 1.5, 0.25),
        ));

        // Mark in nav grid (approximate - walls can be rotated)
        nav_grid.mark_obstacle_world(pos, Vec3::new(6.0, 0.0, 1.0));
    }

    // === CRATES (20-30, shootable) ===
    for _ in 0..25 {
        let x: f32 = rng.random_range(-40.0..40.0);
        let z: f32 = rng.random_range(-40.0..40.0);

        // Avoid spawning too close to center (player spawn)
        if x.abs() < 5.0 && z.abs() < 5.0 {
            continue;
        }

        let pos = Vec3::new(x, 0.75, z);

        commands.spawn((
            Mesh3d(crate_mesh.clone()),
            MeshMaterial3d(crate_material.clone()),
            Transform::from_translation(pos),
            Obstacle::destructible(50.0),
            Shootable,
            RigidBody::Fixed,
            Collider::cuboid(0.75, 0.75, 0.75),
        ));

        nav_grid.mark_obstacle_world(pos, Vec3::new(0.75, 0.0, 0.75));
    }

    // === BARRELS (10-15, shootable) ===
    for _ in 0..12 {
        let x: f32 = rng.random_range(-40.0..40.0);
        let z: f32 = rng.random_range(-40.0..40.0);

        // Avoid spawning too close to center
        if x.abs() < 5.0 && z.abs() < 5.0 {
            continue;
        }

        let pos = Vec3::new(x, 0.75, z);

        commands.spawn((
            Mesh3d(barrel_mesh.clone()),
            MeshMaterial3d(barrel_material.clone()),
            Transform::from_translation(pos),
            Obstacle::destructible(30.0),
            Shootable,
            RigidBody::Fixed,
            Collider::cylinder(0.75, 0.5),
        ));

        nav_grid.mark_obstacle_world(pos, Vec3::new(0.6, 0.0, 0.6));
    }

    // === PILLARS (5-10, indestructible) ===
    let pillar_positions = [
        Vec3::new(-20.0, 2.0, -25.0),
        Vec3::new(20.0, 2.0, -25.0),
        Vec3::new(-20.0, 2.0, 25.0),
        Vec3::new(20.0, 2.0, 25.0),
        Vec3::new(0.0, 2.0, -30.0),
        Vec3::new(0.0, 2.0, 30.0),
        Vec3::new(-35.0, 2.0, 10.0),
        Vec3::new(35.0, 2.0, -10.0),
    ];

    for pos in pillar_positions {
        commands.spawn((
            Mesh3d(pillar_mesh.clone()),
            MeshMaterial3d(pillar_material.clone()),
            Transform::from_translation(pos),
            Obstacle::indestructible(),
            RigidBody::Fixed,
            Collider::cuboid(0.5, 2.0, 0.5),
        ));

        nav_grid.mark_obstacle_world(pos, Vec3::new(0.5, 0.0, 0.5));
    }
}

fn spawn_wall(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    pos: Vec3,
    width: f32,
    height: f32,
    depth: f32,
    _rotation: f32,
    nav_grid: &mut NavGrid,
) {
    // For perimeter walls, use a simple long cuboid mesh
    let perimeter_mesh = Mesh3d(mesh.clone());

    commands.spawn((
        perimeter_mesh,
        MeshMaterial3d(material.clone()),
        Transform::from_translation(pos).with_scale(Vec3::new(width / 10.0, 1.0, depth / 0.5)),
        Obstacle::indestructible(),
        RigidBody::Fixed,
        Collider::cuboid(width / 2.0, height / 2.0, depth / 2.0),
    ));

    nav_grid.mark_obstacle_world(pos, Vec3::new(width / 2.0, 0.0, depth / 2.0));
}
