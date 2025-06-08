use bevy::prelude::*;
use crate::game_state::{GameStats, GameMode, CurrentGameMode, ChallengeTimer, get_score_multiplier, get_magnet_range, UpgradeData};
use crate::Aircraft;

#[derive(Component)]
pub struct Target {
    pub points: u32,
    pub target_type: TargetType,
}

#[derive(Component)]
pub struct Balloon {
    pub base_height: f32,
    pub float_phase: f32,
    pub sway_phase: f32,
}

#[derive(Component)]
pub struct BalloonString;

#[derive(Debug, Clone, Copy)]
pub enum TargetType {
    Normal,
    Golden,
    Speed,
    Time,
    Combo,
}

#[derive(Component)]
pub struct Collectible;

#[derive(Component)]
pub struct ParticleEffect {
    pub lifetime: f32,
    pub velocity: Vec3,
}

#[derive(Event)]
pub struct TargetHitEvent {
    pub position: Vec3,
    pub points: u32,
    pub target_type: TargetType,
}

pub fn spawn_targets_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<&Transform, With<Aircraft>>,
    targets: Query<Entity, With<Target>>,
    game_mode: Res<CurrentGameMode>,
    _time: Res<Time>,
) {
    let target_count = targets.iter().count();
    let max_targets = match game_mode.mode {
        GameMode::TargetHunt => 30,
        GameMode::TimeAttack => 50,
        _ => 40,
    };
    
    if target_count < max_targets {
        if let Ok(aircraft_transform) = query.single() {
            // Spawn targets around the player
            let spawn_distance = 100.0 + fastrand::f32() * 200.0;
            let angle = fastrand::f32() * std::f32::consts::TAU;
            let height = 20.0 + fastrand::f32() * 100.0;
            
            let position = Vec3::new(
                aircraft_transform.translation.x + angle.cos() * spawn_distance,
                height,
                aircraft_transform.translation.z + angle.sin() * spawn_distance,
            );
            
            // Determine target type
            let (target_type, color, points, scale) = match game_mode.mode {
                GameMode::TargetHunt => {
                    if fastrand::f32() < 0.1 {
                        (TargetType::Golden, Color::srgb(1.0, 0.85, 0.0), 500, 2.0)
                    } else {
                        (TargetType::Normal, Color::srgb(0.2, 0.8, 0.2), 100, 1.0)
                    }
                }
                GameMode::Survival => {
                    if fastrand::f32() < 0.2 {
                        (TargetType::Time, Color::srgb(0.2, 0.8, 0.8), 50, 1.5)
                    } else {
                        (TargetType::Normal, Color::srgb(0.2, 0.8, 0.2), 100, 1.0)
                    }
                }
                _ => {
                    let rand = fastrand::f32();
                    if rand < 0.05 {
                        (TargetType::Golden, Color::srgb(1.0, 0.85, 0.0), 500, 2.0)
                    } else if rand < 0.15 {
                        (TargetType::Speed, Color::srgb(0.8, 0.2, 0.8), 200, 1.3)
                    } else if rand < 0.25 {
                        (TargetType::Combo, Color::srgb(0.8, 0.8, 0.2), 150, 1.2)
                    } else {
                        (TargetType::Normal, Color::srgb(0.2, 0.8, 0.2), 100, 1.0)
                    }
                }
            };
            
            // Spawn balloon entity
            let balloon_entity = commands.spawn((
                Transform::from_translation(position),
                Visibility::default(),
                Target {
                    points,
                    target_type,
                },
                Collectible,
                Balloon {
                    base_height: position.y,
                    float_phase: fastrand::f32() * std::f32::consts::TAU,
                    sway_phase: fastrand::f32() * std::f32::consts::TAU,
                },
            )).id();
            
            // Balloon body (slightly elongated sphere)
            let balloon_body = commands.spawn((
                Mesh3d(meshes.add(Sphere::new(1.5 * scale))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive: color.to_linear() * 0.3,
                    metallic: 0.2,
                    perceptual_roughness: 0.3,
                    ..default()
                })),
                Transform::from_scale(Vec3::new(1.0, 1.2, 1.0)),
            )).id();
            
            // Balloon highlight (small sphere for shine effect)
            let highlight = commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.3 * scale))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 1.0, 1.0, 0.6),
                    emissive: Color::WHITE.into(),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::from_xyz(0.5 * scale, 0.8 * scale, 0.5 * scale),
            )).id();
            
            // String segments
            let string_length = 4.0;
            let segments = 5;
            let mut string_entities = vec![];
            
            for i in 0..segments {
                let t = (i as f32) / (segments as f32 - 1.0);
                let y_offset = -1.5 * scale - t * string_length;
                
                let string_segment = commands.spawn((
                    Mesh3d(meshes.add(Cylinder::new(0.02, string_length / segments as f32))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.8, 0.8, 0.8),
                        ..default()
                    })),
                    Transform::from_xyz(0.0, y_offset, 0.0),
                    BalloonString,
                )).id();
                
                string_entities.push(string_segment);
            }
            
            // Build hierarchy
            commands.entity(balloon_entity).add_children(&[balloon_body]);
            commands.entity(balloon_body).add_children(&[highlight]);
            commands.entity(balloon_body).add_children(&string_entities);
        }
    }
}

pub fn collision_detection_system(
    mut commands: Commands,
    mut aircraft_query: Query<(&Transform, &mut crate::enemies::Health), With<Aircraft>>,
    targets_query: Query<(Entity, &Transform, &Target, &Children), With<Collectible>>,
    mut game_stats: ResMut<GameStats>,
    mut challenge_timer: ResMut<ChallengeTimer>,
    _game_mode: Res<CurrentGameMode>,
    upgrades: Res<UpgradeData>,
    mut hit_events: EventWriter<TargetHitEvent>,
) {
    if let Ok((aircraft_transform, mut aircraft_health)) = aircraft_query.single_mut() {
        let magnet_range = get_magnet_range(upgrades.magnet_level);
        let collection_range = 5.0 + magnet_range;
        
        for (entity, target_transform, target, _children) in targets_query.iter() {
            let distance = aircraft_transform.translation.distance(target_transform.translation);
            
            if distance < collection_range {
                // Calculate points with multipliers
                let multiplier = get_score_multiplier(upgrades.multiplier_level);
                let combo_multiplier = 1 + game_stats.combo / 5;
                let total_points = target.points * multiplier * combo_multiplier;
                
                // Update stats
                game_stats.score += total_points;
                game_stats.targets_hit += 1;
                game_stats.combo += 1;
                if game_stats.combo > game_stats.max_combo {
                    game_stats.max_combo = game_stats.combo;
                }
                
                // Apply special effects based on target type
                match target.target_type {
                    TargetType::Time => {
                        challenge_timer.time_remaining += 5.0;
                        if challenge_timer.time_remaining > challenge_timer.total_time {
                            challenge_timer.time_remaining = challenge_timer.total_time;
                        }
                    }
                    TargetType::Speed => {
                        // Speed boost is handled in the aircraft controller
                    }
                    TargetType::Combo => {
                        game_stats.combo += 4; // Extra combo points
                    }
                    _ => {}
                }
                
                // Heal player when collecting balloons
                let heal_amount = match target.target_type {
                    TargetType::Golden => 20.0,  // Golden balloons heal more
                    TargetType::Time => 15.0,    // Time balloons heal moderately
                    _ => 10.0,                   // Normal balloons heal less
                };
                
                aircraft_health.current = (aircraft_health.current + heal_amount).min(aircraft_health.max);
                
                // Send hit event
                hit_events.write(TargetHitEvent {
                    position: target_transform.translation,
                    points: total_points,
                    target_type: target.target_type,
                });
                
                // Remove the balloon and all its children
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn magnet_effect_system(
    mut targets_query: Query<(&mut Transform, &Target, &Balloon), With<Collectible>>,
    aircraft_query: Query<&Transform, (With<Aircraft>, Without<Target>)>,
    upgrades: Res<UpgradeData>,
    time: Res<Time>,
) {
    if upgrades.magnet_level == 0 {
        return;
    }
    
    if let Ok(aircraft_transform) = aircraft_query.single() {
        let magnet_range = get_magnet_range(upgrades.magnet_level) + 20.0;
        let magnet_strength = 15.0 * upgrades.magnet_level as f32;
        
        for (mut target_transform, _, _balloon) in targets_query.iter_mut() {
            let distance = aircraft_transform.translation.distance(target_transform.translation);
            
            if distance < magnet_range {
                let direction = (aircraft_transform.translation - target_transform.translation).normalize();
                let force = magnet_strength * (1.0 - distance / magnet_range);
                target_transform.translation += direction * force * time.delta_secs();
            }
        }
    }
}

pub fn particle_system(
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Transform, &mut ParticleEffect)>,
    time: Res<Time>,
) {
    for (entity, mut transform, mut particle) in particles.iter_mut() {
        particle.lifetime -= time.delta_secs();
        
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            transform.translation += particle.velocity * time.delta_secs();
            particle.velocity.y -= 9.8 * time.delta_secs(); // Gravity
            
            // Fade out
            transform.scale = Vec3::splat(particle.lifetime / 1.0);
        }
    }
}

pub fn spawn_hit_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut hit_events: EventReader<TargetHitEvent>,
) {
    for event in hit_events.read() {
        // Spawn multiple particles
        for i in 0..15 {
            let velocity = Vec3::new(
                (fastrand::f32() - 0.5) * 20.0,
                fastrand::f32() * 15.0 + 5.0,
                (fastrand::f32() - 0.5) * 20.0,
            );
            
            let color = match event.target_type {
                TargetType::Golden => Color::srgb(1.0, 0.85, 0.0),
                TargetType::Speed => Color::srgb(0.8, 0.2, 0.8),
                TargetType::Time => Color::srgb(0.2, 0.8, 0.8),
                TargetType::Combo => Color::srgb(0.8, 0.8, 0.2),
                TargetType::Normal => Color::srgb(0.2, 0.8, 0.2),
            };
            
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.3))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive: color.to_linear() * 2.0,
                    ..default()
                })),
                Transform::from_translation(event.position),
                ParticleEffect {
                    lifetime: 1.0,
                    velocity,
                },
            ));
            
            // Add green healing particles
            if i < 5 {
                let heal_velocity = Vec3::new(
                    (fastrand::f32() - 0.5) * 10.0,
                    fastrand::f32() * 5.0 + 10.0,
                    (fastrand::f32() - 0.5) * 10.0,
                );
                
                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.2))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.2, 1.0, 0.2),
                        emissive: Color::srgb(0.2, 1.0, 0.2).into(),
                        ..default()
                    })),
                    Transform::from_translation(event.position),
                    ParticleEffect {
                        lifetime: 1.5,
                        velocity: heal_velocity,
                    },
                ));
            }
        }
    }
}

pub fn combo_timeout_system(
    mut game_stats: ResMut<GameStats>,
    time: Res<Time>,
    mut combo_timer: Local<f32>,
) {
    if game_stats.combo > 0 {
        *combo_timer += time.delta_secs();
        
        if *combo_timer > 3.0 {
            game_stats.combo = 0;
            *combo_timer = 0.0;
        }
    } else {
        *combo_timer = 0.0;
    }
}

pub fn animate_targets(
    mut balloons: Query<(&mut Transform, &Balloon, &Target), With<Collectible>>,
    mut strings: Query<&mut Transform, (With<BalloonString>, Without<Balloon>, Without<Collectible>)>,
    time: Res<Time>,
) {
    for (mut transform, balloon, target) in balloons.iter_mut() {
        let elapsed = time.elapsed_secs();
        
        // Floating motion - gentle up and down
        let float_amount = match target.target_type {
            TargetType::Golden => 3.0,
            TargetType::Speed => 2.5,
            _ => 2.0,
        };
        let float_offset = (elapsed * 0.8 + balloon.float_phase).sin() * float_amount;
        
        // Swaying motion - side to side
        let sway_amount = 1.5;
        let sway_x = (elapsed * 0.6 + balloon.sway_phase).sin() * sway_amount;
        let sway_z = (elapsed * 0.7 + balloon.sway_phase + 1.0).sin() * sway_amount * 0.7;
        
        // Apply motion
        transform.translation.y = balloon.base_height + float_offset;
        transform.translation.x += sway_x * time.delta_secs();
        transform.translation.z += sway_z * time.delta_secs();
        
        // Gentle rotation
        transform.rotate_y(time.delta_secs() * 0.5);
        
        // Tilt based on movement
        let tilt_amount = 0.1;
        transform.rotation = Quat::from_euler(
            EulerRot::XYZ,
            sway_z * tilt_amount,
            transform.rotation.to_euler(EulerRot::XYZ).1,
            -sway_x * tilt_amount,
        );
    }
    
    // Animate strings with physics-like motion
    for mut string_transform in strings.iter_mut() {
        // Add slight sway to strings
        let sway = (time.elapsed_secs() * 2.0).sin() * 0.02;
        string_transform.rotation = Quat::from_rotation_z(sway);
    }
}