use bevy::prelude::*;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::input::mouse::MouseMotion;
use bevy::window::{WindowMode, PrimaryWindow};
use bevy_egui::EguiPlugin;

mod game_state;
mod ui;
mod targets;
mod enemies;
mod powerups;
mod tests;

use game_state::*;
use ui::*;
use targets::*;
use enemies::*;
use powerups::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Sky Hunter".to_string(),
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }), EguiPlugin { enable_multipass_for_primary_context: false }))
        .init_state::<GameState>()
        .init_resource::<CurrentGameMode>()
        .init_resource::<GameStats>()
        .init_resource::<ChallengeTimer>()
        .init_resource::<UpgradeData>()
        .init_resource::<ActivePowerUps>()
        .add_event::<TargetHitEvent>()
        .add_event::<EnemyDestroyedEvent>()
        .add_systems(Startup, setup_menu_camera)
        .add_systems(OnEnter(GameState::Playing), (setup_game, capture_mouse))
        .add_systems(OnExit(GameState::Playing), release_mouse)
        .add_systems(OnEnter(GameState::MainMenu), (cleanup_game_entities, cleanup_game_stats))
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
                handle_escape_key,
            ).run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                spawn_enemies_system,
                enemy_ai_system,
                enemy_shooting_system,
                player_shooting_system,
                update_bullets_system,
                bullet_collision_system,
                player_damage_system,
                player_enemy_collision_system,
                spawn_explosion_particles,
                spawn_powerups_system,
                animate_powerups,
                collect_powerups_system,
                update_powerup_effects,
                cleanup_expired_powerups,
                update_shield_visual,
            ).run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::Paused), release_mouse)
        .add_systems(OnExit(GameState::Paused), capture_mouse)
        .add_systems(
            Update,
            (pause_menu, toggle_fullscreen).run_if(in_state(GameState::Paused)),
        )
        .add_systems(
            Update,
            (game_over_screen, save_coins).run_if(in_state(GameState::GameOver)),
        )
        .add_systems(OnExit(GameState::GameOver), cleanup_game)
        .add_systems(
            Update,
            upgrade_shop_ui.run_if(in_state(GameState::UpgradeShop)),
        )
        .run();
}

#[derive(Component)]
pub struct Aircraft {
    speed: f32,
    roll_speed: f32,
    current_roll: f32,
    target_roll: f32,
    boost_timer: f32,
}

#[derive(Component)]
struct FlightCamera {
    shake_amount: f32,
    shake_timer: f32,
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
    for camera in menu_camera.iter() {
        commands.entity(camera).despawn();
    }
    
    // Ground plane with improved material
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(3000.0, 3000.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.4, 0.2),
            metallic: 0.0,
            perceptual_roughness: 0.9,
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
            roll_speed: 1.5 * maneuverability_multiplier,
            current_roll: 0.0,
            target_roll: 0.0,
            boost_timer: 0.0,
        },
        Health {
            current: 100.0,
            max: 100.0,
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
        Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.5, 0.7, 1.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 55.0, 15.0)
            .looking_at(Vec3::new(0.0, 50.0, 0.0), Vec3::Y),
        FlightCamera {
            shake_amount: 0.0,
            shake_timer: 0.0,
        },
        GameEntity,
    ));
    
    // Sun light
    commands.spawn((
        DirectionalLight {
            illuminance: 20000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.3, 0.0)),
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            maximum_distance: 1000.0,
            ..default()
        }.build(),
        GameEntity,
    ));
    
    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.8, 0.85, 1.0),
        brightness: 600.0,
        affects_lightmapped_meshes: false,
    });
    
    // Add some environmental decoration - trees
    for _ in 0..50 {
        let x = (fastrand::f32() - 0.5) * 1000.0;
        let z = (fastrand::f32() - 0.5) * 1000.0;
        let height = 10.0 + fastrand::f32() * 15.0;
        
        // Tree trunk
        commands.spawn((
            Mesh3d(meshes.add(Cylinder::new(2.0, height))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.3, 0.2),
                perceptual_roughness: 0.9,
                ..default()
            })),
            Transform::from_xyz(x, height / 2.0, z),
            GameEntity,
        ));
        
        // Tree leaves
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(8.0 + fastrand::f32() * 4.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.6, 0.2),
                perceptual_roughness: 0.8,
                ..default()
            })),
            Transform::from_xyz(x, height + 5.0, z),
            GameEntity,
        ));
    }
    
    // Add fluffy clouds
    for _ in 0..30 {
        let x = (fastrand::f32() - 0.5) * 2000.0;
        let z = (fastrand::f32() - 0.5) * 2000.0;
        let y = 150.0 + fastrand::f32() * 150.0;
        
        // Create cloud cluster for fluffiness
        let cloud_center = Vec3::new(x, y, z);
        let cloud_parts = 3 + fastrand::usize(..4);
        
        for _ in 0..cloud_parts {
            let offset = Vec3::new(
                (fastrand::f32() - 0.5) * 40.0,
                (fastrand::f32() - 0.5) * 20.0,
                (fastrand::f32() - 0.5) * 40.0,
            );
            
            let size = 20.0 + fastrand::f32() * 30.0;
            let opacity = 0.4 + fastrand::f32() * 0.2;
            
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(size))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 1.0, 1.0, opacity),
                    alpha_mode: AlphaMode::Blend,
                    perceptual_roughness: 1.0,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_translation(cloud_center + offset)
                    .with_scale(Vec3::new(1.5, 0.7, 1.5)), // Flatten clouds
                GameEntity,
            ));
        }
    }
}

fn cleanup_game(
    mut commands: Commands,
    query: Query<Entity, With<GameEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Recreate menu camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 100.0, 200.0)
            .looking_at(Vec3::new(0.0, 50.0, 0.0), Vec3::Y),
        MenuCamera,
    ));
}

fn cleanup_game_stats(
    mut game_stats: ResMut<GameStats>,
    mut active_powerups: ResMut<ActivePowerUps>,
) {
    // Reset per-game stats but keep persistent ones
    game_stats.score = 0;
    game_stats.combo = 0;
    game_stats.targets_hit = 0;
    game_stats.time_played = 0.0;
    
    // Reset active powerups
    active_powerups.reset();
}

fn flight_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Aircraft)>,
    mut camera_query: Query<(&mut Transform, &mut FlightCamera), Without<Aircraft>>,
    game_state: Res<State<GameState>>,
    mut mouse_delta: Local<Vec2>,
    mut motion_events: EventReader<MouseMotion>,
    active_powerups: Res<ActivePowerUps>,
) {
    if *game_state != GameState::Playing {
        return;
    }
    
    // Accumulate mouse movement
    for event in motion_events.read() {
        *mouse_delta += event.delta;
    }
    
    for (mut transform, mut aircraft) in query.iter_mut() {
        let delta = time.delta_secs();
        
        // Enhanced mouse controls with improved responsiveness
        let sensitivity = 0.001; // Slightly increased for better control
        if mouse_delta.length() > 0.0 {
            // Smooth mouse input with adaptive sensitivity
            let mouse_speed = mouse_delta.length();
            let adaptive_sensitivity = sensitivity * (1.0 + mouse_speed * 0.0001).min(2.0);
            
            let smoothed_x = mouse_delta.x.clamp(-200.0, 200.0);
            let smoothed_y = mouse_delta.y.clamp(-200.0, 200.0);
            
            // Yaw (left/right mouse movement) with momentum
            let yaw_amount = -smoothed_x * adaptive_sensitivity;
            transform.rotate_y(yaw_amount);
            
            // Pitch (up/down mouse movement) with realistic limits
            let pitch_amount = -smoothed_y * adaptive_sensitivity;
            let current_pitch = transform.rotation.to_euler(EulerRot::YXZ).1;
            let new_pitch = (current_pitch + pitch_amount).clamp(-1.0, 0.8); // Asymmetric limits
            transform.rotation = Quat::from_euler(
                EulerRot::YXZ,
                transform.rotation.to_euler(EulerRot::YXZ).0,
                new_pitch,
                transform.rotation.to_euler(EulerRot::YXZ).2,
            );
            
            // Auto-roll based on yaw for realistic banking
            aircraft.target_roll = -smoothed_x * 0.015 * (1.0 + aircraft.speed / 100.0).min(2.0);
            
            // Reset mouse delta with decay for smoother control
            *mouse_delta *= 0.2;
        } else {
            aircraft.target_roll *= 0.95; // Gradual return to neutral
        }
        
        // Manual roll controls (A/D) with improved banking
        if keyboard_input.pressed(KeyCode::KeyA) {
            aircraft.target_roll = 0.7;
            transform.rotate_y(aircraft.roll_speed * 0.3 * delta); // Banking affects turn rate
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            aircraft.target_roll = -0.7;
            transform.rotate_y(-aircraft.roll_speed * 0.3 * delta);
        }
        
        // Advanced roll physics
        aircraft.current_roll = aircraft.current_roll.lerp(aircraft.target_roll, delta * 4.0);
        transform.rotate_local_z(aircraft.current_roll * aircraft.roll_speed * delta);
        
        // Speed controls with acceleration/deceleration
        let base_speed = aircraft.speed * active_powerups.speed_multiplier;
        let mut target_speed = base_speed;
        
        // Throttle controls
        if keyboard_input.pressed(KeyCode::KeyW) {
            target_speed = base_speed * 1.8;
            aircraft.boost_timer = 0.05; // Light afterburner effect
        } else if keyboard_input.pressed(KeyCode::KeyS) {
            target_speed = base_speed * 0.5;
        }
        
        // Boost with energy management
        if keyboard_input.pressed(KeyCode::Space) && aircraft.boost_timer <= 0.0 {
            target_speed = base_speed * 3.0;
            aircraft.boost_timer = 0.2;
        }
        
        // Smooth speed transitions
        let current_speed = if aircraft.boost_timer > 0.0 {
            target_speed
        } else {
            aircraft.speed + (target_speed - aircraft.speed) * delta * 3.0
        };
        
        // Update boost timer
        if aircraft.boost_timer > 0.0 {
            aircraft.boost_timer -= delta;
        }
        
        // Advanced movement with lift simulation
        let forward = transform.forward();
        let velocity = forward * current_speed;
        
        // Add lift based on speed and pitch
        let pitch_angle = transform.rotation.to_euler(EulerRot::YXZ).1;
        let lift_factor = (current_speed / base_speed).min(2.0) * pitch_angle.sin();
        let lift = Vec3::Y * lift_factor * 10.0;
        
        transform.translation += (velocity + lift) * delta;
        
        // Altitude management with ground effect
        let ground_height = 5.0;
        let effect_height = 20.0;
        if transform.translation.y < effect_height {
            let ground_effect = 1.0 - (transform.translation.y - ground_height) / (effect_height - ground_height);
            let upward_force = ground_effect.max(0.0) * 50.0;
            transform.translation.y += upward_force * delta;
            
            // Auto-level when very low
            if transform.translation.y < ground_height + 5.0 {
                let level_rotation = transform.rotation.slerp(
                    Quat::from_rotation_y(transform.rotation.to_euler(EulerRot::YXZ).0),
                    delta * 2.0
                );
                transform.rotation = level_rotation;
            }
        }
        
        // Prevent going below ground
        transform.translation.y = transform.translation.y.max(ground_height);
        
        // Update camera with cinematic movement
        if let Ok((mut camera_transform, mut camera)) = camera_query.single_mut() {
            // Dynamic camera positioning
            let speed_ratio = current_speed / base_speed;
            let base_offset = Vec3::new(0.0, 8.0, 20.0);
            
            // Pull camera back when boosting
            let distance_multiplier = 1.0 + (speed_ratio - 1.0).max(0.0) * 0.5;
            let height_multiplier = 1.0 - (aircraft.current_roll.abs() * 0.2); // Lower when banking
            
            let camera_offset = base_offset * Vec3::new(1.0, height_multiplier, distance_multiplier);
            let rotated_offset = transform.rotation * camera_offset;
            let target_pos = transform.translation + rotated_offset;
            
            // Smooth camera follow with lag
            let follow_speed = Vec3::new(6.0, 4.0, 6.0) * (2.0 - speed_ratio * 0.5).max(0.5);
            camera_transform.translation.x = camera_transform.translation.x.lerp(target_pos.x, delta * follow_speed.x);
            camera_transform.translation.y = camera_transform.translation.y.lerp(target_pos.y, delta * follow_speed.y);
            camera_transform.translation.z = camera_transform.translation.z.lerp(target_pos.z, delta * follow_speed.z);
            
            // Camera shake effects
            if aircraft.boost_timer > 0.0 {
                camera.shake_amount = 3.0 * aircraft.boost_timer / 0.2;
                camera.shake_timer = aircraft.boost_timer;
            }
            
            // Apply camera shake with turbulence
            if camera.shake_timer > 0.0 {
                camera.shake_timer -= delta;
                let turbulence = time.elapsed_secs() * 15.0;
                let shake_offset = Vec3::new(
                    turbulence.sin() * camera.shake_amount * 0.5,
                    turbulence.cos() * camera.shake_amount * 0.3,
                    0.0
                );
                camera_transform.translation += shake_offset;
            }
            
            // Look ahead with predictive targeting
            let velocity_prediction = velocity * 0.2;
            let look_target = transform.translation + forward * 20.0 + velocity_prediction;
            camera_transform.look_at(look_target, Vec3::Y);
            
            // Dynamic camera roll
            let camera_roll = aircraft.current_roll * 0.4 * (1.0 - speed_ratio * 0.2).max(0.3);
            camera_transform.rotate_z(camera_roll);
        }
    }
}


fn update_challenge_timer(
    mut timer: ResMut<ChallengeTimer>,
    game_mode: Res<CurrentGameMode>,
    mut game_stats: ResMut<GameStats>,
    time: Res<Time>,
) {
    // Update time played
    game_stats.time_played += time.delta_secs();
    
    // Increase difficulty over time
    let difficulty_increase_rate = 0.1; // 10% per minute
    game_stats.difficulty_level = 1.0 + (game_stats.time_played / 60.0) * difficulty_increase_rate;
    
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
        // Removed unused variable
        
        // Base color changes with boost
        let (base_color, emissive_strength, trail_size) = if keyboard_input.pressed(KeyCode::Space) {
            (Color::srgb(1.0, 0.4, 0.1), 3.0, 0.5) // Orange for boost
        } else if keyboard_input.pressed(KeyCode::KeyW) {
            (Color::srgb(0.4, 0.7, 1.0), 2.0, 0.4) // Bright blue for speed
        } else {
            (Color::srgb(0.2, 0.5, 0.9), 1.5, 0.3) // Blue for normal
        };
        
        // Spawn two trails for each engine
        for offset in [Vec3::new(-3.0, -0.5, 3.0), Vec3::new(3.0, -0.5, 3.0)] {
            let trail_offset = transform.rotation * offset;
            let trail_pos = transform.translation + trail_offset;
            
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(trail_size))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color,
                    emissive: base_color.to_linear() * emissive_strength,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_translation(trail_pos),
                EngineTrail {
                    lifetime: 0.0,
                    max_lifetime: if keyboard_input.pressed(KeyCode::Space) { 0.8 } else { 0.5 },
                },
                GameEntity,
            ));
        }
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

fn handle_escape_key(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        game_state.set(GameState::Paused);
    }
}

fn toggle_fullscreen(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keyboard_input.just_pressed(KeyCode::F11) {
        if let Ok(mut window) = windows.single_mut() {
            window.mode = match window.mode {
                WindowMode::BorderlessFullscreen(_) => WindowMode::Windowed,
                _ => WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
            };
        }
    }
}

fn capture_mouse(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = windows.single_mut() {
        window.cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

fn release_mouse(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = windows.single_mut() {
        window.cursor_options.grab_mode = bevy::window::CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

fn cleanup_game_entities(
    mut commands: Commands,
    query: Query<Entity, With<GameEntity>>,
    cameras: Query<Entity, With<MenuCamera>>,
) {
    // Remove all game entities
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
    
    // Recreate menu camera if it doesn't exist
    if cameras.is_empty() {
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 100.0, 200.0)
                .looking_at(Vec3::new(0.0, 50.0, 0.0), Vec3::Y),
            MenuCamera,
        ));
    }
}

fn player_enemy_collision_system(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut enemies::Health), With<Aircraft>>,
    enemy_query: Query<(Entity, &Transform, &enemies::Enemy), Without<Aircraft>>,
    mut destroyed_events: EventWriter<enemies::EnemyDestroyedEvent>,
    mut game_stats: ResMut<GameStats>,
    mut camera_query: Query<&mut FlightCamera>,
) {
    if let Ok((player_transform, mut player_health)) = player_query.single_mut() {
        for (enemy_entity, enemy_transform, enemy) in enemy_query.iter() {
            let distance = player_transform.translation.distance(enemy_transform.translation);
            
            // Collision radius based on enemy type
            let collision_radius = match enemy.enemy_type {
                enemies::EnemyType::Bomber => 8.0,
                enemies::EnemyType::Fighter => 6.0,
                enemies::EnemyType::Ace => 5.0,
            };
            
            if distance < collision_radius {
                // Collision damage
                let collision_damage = match enemy.enemy_type {
                    enemies::EnemyType::Bomber => 40.0,
                    enemies::EnemyType::Fighter => 30.0,
                    enemies::EnemyType::Ace => 25.0,
                };
                
                player_health.current = (player_health.current - collision_damage).max(0.0);
                
                // Award partial points for ramming
                let ram_points = match enemy.enemy_type {
                    enemies::EnemyType::Fighter => 25,
                    enemies::EnemyType::Bomber => 50,
                    enemies::EnemyType::Ace => 100,
                };
                game_stats.score += ram_points;
                
                // Send destroyed event
                destroyed_events.write(enemies::EnemyDestroyedEvent {
                    position: enemy_transform.translation,
                    enemy_type: enemy.enemy_type,
                });
                
                // Remove enemy
                commands.entity(enemy_entity).despawn();
                
                // Camera shake on collision
                if let Ok(mut camera) = camera_query.single_mut() {
                    camera.shake_amount = 5.0;
                    camera.shake_timer = 0.3;
                }
            }
        }
    }
}