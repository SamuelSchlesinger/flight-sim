use bevy::prelude::*;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::input::mouse::MouseMotion;
use bevy_egui::EguiPlugin;

mod game_state;
mod ui;
mod targets;

use game_state::*;
use ui::*;
use targets::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin { enable_multipass_for_primary_context: false }))
        .init_state::<GameState>()
        .init_resource::<CurrentGameMode>()
        .init_resource::<GameStats>()
        .init_resource::<ChallengeTimer>()
        .init_resource::<UpgradeData>()
        .add_event::<TargetHitEvent>()
        .add_systems(Startup, setup_menu_camera)
        .add_systems(OnEnter(GameState::Playing), setup_game)
        .add_systems(OnExit(GameState::Playing), cleanup_game)
        .add_systems(
            Update,
            (
                main_menu_ui,
                update_high_score,
            ).run_if(in_state(GameState::MainMenu)),
        )
        .add_systems(
            Update,
            (
                flight_controls,
                spawn_engine_trails,
                update_engine_trails,
                spawn_targets_system,
                collision_detection_system,
                magnet_effect_system,
                animate_targets,
                particle_system,
                spawn_hit_particles,
                combo_timeout_system,
                update_challenge_timer,
                check_game_over,
                game_hud,
            ).run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            pause_menu.run_if(in_state(GameState::Paused)),
        )
        .add_systems(
            Update,
            (game_over_screen, save_coins).run_if(in_state(GameState::GameOver)),
        )
        .add_systems(
            Update,
            upgrade_shop_ui.run_if(in_state(GameState::UpgradeShop)),
        )
        .run();
}

#[derive(Component)]
pub struct Aircraft {
    speed: f32,
    pitch_speed: f32,
    roll_speed: f32,
    yaw_speed: f32,
}

#[derive(Component)]
struct FlightCamera {
    sensitivity: f32,
}

#[derive(Component)]
struct GameEntity;

#[derive(Component)]
struct MenuCamera;

#[derive(Component)]
struct EngineTrail {
    lifetime: f32,
    max_lifetime: f32,
}

fn setup_menu_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 100.0, 200.0)
            .looking_at(Vec3::new(0.0, 50.0, 0.0), Vec3::Y),
        MenuCamera,
    ));
}

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    menu_camera: Query<Entity, With<MenuCamera>>,
    upgrades: Res<UpgradeData>,
) {
    // Remove menu camera
    if let Ok(camera) = menu_camera.single() {
        commands.entity(camera).despawn();
    }
    
    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(2000.0, 2000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GameEntity,
    ));
    
    // Calculate upgrade bonuses
    let speed_multiplier = get_speed_bonus(upgrades.speed_level);
    let maneuverability_multiplier = get_maneuverability_bonus(upgrades.maneuverability_level);
    
    // Aircraft (player) - parent entity
    let aircraft_entity = commands.spawn((
        Transform::from_xyz(0.0, 50.0, 0.0),
        Visibility::default(),
        Aircraft {
            speed: 50.0 * speed_multiplier,
            pitch_speed: 1.0 * maneuverability_multiplier,
            roll_speed: 1.5 * maneuverability_multiplier,
            yaw_speed: 1.0 * maneuverability_multiplier,
        },
        GameEntity,
    )).id();
    
    // Fuselage
    let fuselage = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.5, 1.0, 4.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    // Nose cone (snout)
    let nose = commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.5, 1.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.6, 0.15, 0.15),
            metallic: 0.9,
            perceptual_roughness: 0.2,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, -2.75)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::PI / 2.0)),
    )).id();
    
    // Left wing
    let left_wing = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(6.0, 0.2, 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.3, 0.3),
            metallic: 0.7,
            perceptual_roughness: 0.4,
            ..default()
        })),
        Transform::from_xyz(-4.0, 0.0, 0.5),
    )).id();
    
    // Right wing
    let right_wing = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(6.0, 0.2, 2.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.9, 0.3, 0.3),
            metallic: 0.7,
            perceptual_roughness: 0.4,
            ..default()
        })),
        Transform::from_xyz(4.0, 0.0, 0.5),
    )).id();
    
    // Tail fin
    let tail_fin = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 2.0, 1.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.2, 0.2),
            metallic: 0.6,
            perceptual_roughness: 0.5,
            ..default()
        })),
        Transform::from_xyz(0.0, 1.0, 2.5),
    )).id();
    
    // Horizontal stabilizer
    let h_stabilizer = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(3.0, 0.15, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.2, 0.2),
            metallic: 0.6,
            perceptual_roughness: 0.5,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 2.5),
    )).id();
    
    // Build the aircraft hierarchy
    commands.entity(aircraft_entity).add_children(&[
        fuselage,
        nose,
        left_wing,
        right_wing,
        tail_fin,
        h_stabilizer,
    ]);
    
    // Camera attached to aircraft
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 55.0, 15.0)
            .looking_at(Vec3::new(0.0, 50.0, 0.0), Vec3::Y),
        FlightCamera {
            sensitivity: 0.002,
        },
        GameEntity,
    ));
    
    // Sun light
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.0, -0.5, 0.0)),
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            maximum_distance: 1000.0,
            ..default()
        }.build(),
        GameEntity,
    ));
    
    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 400.0,
        affects_lightmapped_meshes: false,
    });
}

fn cleanup_game(
    mut commands: Commands,
    query: Query<Entity, With<GameEntity>>,
    mut game_stats: ResMut<GameStats>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Reset per-game stats but keep persistent ones
    game_stats.score = 0;
    game_stats.combo = 0;
    game_stats.targets_hit = 0;
    game_stats.time_played = 0.0;
    
    // Recreate menu camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 100.0, 200.0)
            .looking_at(Vec3::new(0.0, 50.0, 0.0), Vec3::Y),
        MenuCamera,
    ));
}

fn flight_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Aircraft)>,
    mut camera_query: Query<&mut Transform, (With<Camera3d>, Without<Aircraft>)>,
    game_state: Res<State<GameState>>,
    mut mouse_delta: Local<Vec2>,
    mut motion_events: EventReader<MouseMotion>,
) {
    if *game_state != GameState::Playing {
        return;
    }
    
    // Accumulate mouse movement
    for event in motion_events.read() {
        *mouse_delta += event.delta;
    }
    
    for (mut transform, aircraft) in query.iter_mut() {
        let delta = time.delta_secs();
        
        // Mouse controls for pitch and yaw (more intuitive for flying)
        let sensitivity = 0.001;
        if mouse_delta.length() > 0.0 {
            // Yaw (left/right mouse movement)
            transform.rotate_y(-mouse_delta.x * sensitivity);
            
            // Pitch (up/down mouse movement)
            transform.rotate_local_x(-mouse_delta.y * sensitivity);
            
            // Reset mouse delta
            *mouse_delta = Vec2::ZERO;
        }
        
        // Roll controls (A/D) - banking turns
        if keyboard_input.pressed(KeyCode::KeyA) {
            transform.rotate_local_z(aircraft.roll_speed * delta);
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            transform.rotate_local_z(-aircraft.roll_speed * delta);
        }
        
        // Speed controls
        let mut current_speed = aircraft.speed;
        
        // W for speed up, S for slow down
        if keyboard_input.pressed(KeyCode::KeyW) {
            current_speed *= 1.5;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            current_speed *= 0.7;
        }
        
        // Space for boost
        if keyboard_input.pressed(KeyCode::Space) {
            current_speed *= 2.5;
        }
        
        // Move forward
        let forward = transform.forward();
        transform.translation += forward * current_speed * delta;
        
        // Keep aircraft above ground with smooth correction
        let min_height = 10.0;
        if transform.translation.y < min_height {
            transform.translation.y = transform.translation.y.lerp(min_height, delta * 5.0);
        }
        
        // Update camera to follow aircraft with smoother movement
        if let Ok(mut camera_transform) = camera_query.single_mut() {
            let camera_offset = Vec3::new(0.0, 8.0, 20.0);
            let rotated_offset = transform.rotation * camera_offset;
            let target_pos = transform.translation + rotated_offset;
            
            // Smooth camera follow
            camera_transform.translation = camera_transform.translation.lerp(target_pos, delta * 8.0);
            
            // Look slightly ahead of the aircraft
            let look_ahead = transform.translation + forward * 10.0;
            camera_transform.look_at(look_ahead, Vec3::Y);
        }
    }
}


fn update_challenge_timer(
    mut timer: ResMut<ChallengeTimer>,
    game_mode: Res<CurrentGameMode>,
    time: Res<Time>,
) {
    match game_mode.mode {
        GameMode::TimeAttack | GameMode::Survival | GameMode::RaceTheClock => {
            timer.time_remaining -= time.delta_secs();
            if timer.time_remaining < 0.0 {
                timer.time_remaining = 0.0;
            }
        }
        _ => {}
    }
}

fn check_game_over(
    timer: Res<ChallengeTimer>,
    game_mode: Res<CurrentGameMode>,
    game_stats: Res<GameStats>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    match game_mode.mode {
        GameMode::TimeAttack | GameMode::Survival => {
            if timer.time_remaining <= 0.0 {
                game_state.set(GameState::GameOver);
            }
        }
        GameMode::RaceTheClock => {
            if timer.time_remaining <= 0.0 || game_stats.targets_hit >= 50 {
                game_state.set(GameState::GameOver);
            }
        }
        _ => {}
    }
}

fn update_high_score(mut game_stats: ResMut<GameStats>) {
    if game_stats.score > game_stats.high_score {
        game_stats.high_score = game_stats.score;
    }
}

fn save_coins(mut game_stats: ResMut<GameStats>, mut saved: Local<bool>) {
    if !*saved {
        let coins_earned = game_stats.score / 100;
        game_stats.coins += coins_earned;
        *saved = true;
    }
}

fn spawn_engine_trails(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    aircraft_query: Query<&Transform, With<Aircraft>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
) {
    *spawn_timer += time.delta_secs();
    
    // Spawn trails more frequently when boosting
    let spawn_interval = if keyboard_input.pressed(KeyCode::Space) {
        0.02
    } else {
        0.05
    };
    
    if *spawn_timer < spawn_interval {
        return;
    }
    
    *spawn_timer = 0.0;
    
    for transform in aircraft_query.iter() {
        // Spawn trail particles behind the aircraft
        let trail_offset = transform.rotation * Vec3::new(0.0, -0.5, 3.0);
        let trail_pos = transform.translation + trail_offset;
        
        // Base color changes with boost
        let base_color = if keyboard_input.pressed(KeyCode::Space) {
            Color::srgb(1.0, 0.5, 0.0) // Orange for boost
        } else {
            Color::srgb(0.3, 0.6, 1.0) // Blue for normal
        };
        
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.3))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color,
                emissive: base_color.into(),
                ..default()
            })),
            Transform::from_translation(trail_pos),
            EngineTrail {
                lifetime: 0.0,
                max_lifetime: 0.5,
            },
            GameEntity,
        ));
    }
}

fn update_engine_trails(
    mut commands: Commands,
    mut trail_query: Query<(Entity, &mut Transform, &mut EngineTrail, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut trail, material_handle) in trail_query.iter_mut() {
        trail.lifetime += time.delta_secs();
        
        let lifetime_ratio = trail.lifetime / trail.max_lifetime;
        
        // Fade out and shrink
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let alpha = 1.0 - lifetime_ratio;
            material.base_color.set_alpha(alpha);
            
            // Scale down
            let scale = 1.0 - (lifetime_ratio * 0.8);
            transform.scale = Vec3::splat(scale);
        }
        
        // Remove when lifetime expires
        if trail.lifetime >= trail.max_lifetime {
            commands.entity(entity).despawn();
        }
    }
}