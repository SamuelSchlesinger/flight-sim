use bevy::prelude::*;
use crate::{Aircraft, GameEntity, game_state::GameStats, FlightCamera};

#[derive(Component)]
pub struct PowerUp {
    pub power_type: PowerUpType,
    pub lifetime: f32,
    pub bob_phase: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PowerUpType {
    RapidFire,
    Shield,
    SpeedBoost,
    HealthPack,
    EnergyRecharge,
    TripleShot,
    HomingMissiles,
}

#[derive(Component)]
pub struct PowerUpEffect {
    pub effect_type: PowerUpType,
    pub duration: f32,
    pub remaining: f32,
}

#[derive(Resource, Default)]
pub struct ActivePowerUps {
    pub rapid_fire: bool,
    pub shield: bool,
    pub triple_shot: bool,
    pub homing_missiles: bool,
    pub speed_multiplier: f32,
}

impl ActivePowerUps {
    pub fn reset(&mut self) {
        self.rapid_fire = false;
        self.shield = false;
        self.triple_shot = false;
        self.homing_missiles = false;
        self.speed_multiplier = 1.0;
    }
}

pub fn spawn_powerups_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<Aircraft>>,
    powerups: Query<Entity, With<PowerUp>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
) {
    let powerup_count = powerups.iter().count();
    let max_powerups = 3;
    
    *spawn_timer += time.delta_secs();
    
    if powerup_count < max_powerups && *spawn_timer > 10.0 {
        *spawn_timer = 0.0;
        
        if let Ok(player_transform) = player_query.single() {
            // Spawn powerup at distance from player
            let spawn_distance = 100.0 + fastrand::f32() * 150.0;
            let angle = fastrand::f32() * std::f32::consts::TAU;
            let height = 30.0 + fastrand::f32() * 80.0;
            
            let position = Vec3::new(
                player_transform.translation.x + angle.cos() * spawn_distance,
                height,
                player_transform.translation.z + angle.sin() * spawn_distance,
            );
            
            // Random powerup type
            let power_type = match fastrand::u8(0..7) {
                0 => PowerUpType::RapidFire,
                1 => PowerUpType::Shield,
                2 => PowerUpType::SpeedBoost,
                3 => PowerUpType::HealthPack,
                4 => PowerUpType::EnergyRecharge,
                5 => PowerUpType::TripleShot,
                _ => PowerUpType::HomingMissiles,
            };
            
            let (color, emissive_color) = match power_type {
                PowerUpType::RapidFire => (Color::srgb(1.0, 0.5, 0.0), Color::srgb(1.0, 0.3, 0.0)),
                PowerUpType::Shield => (Color::srgb(0.0, 0.5, 1.0), Color::srgb(0.0, 0.3, 1.0)),
                PowerUpType::SpeedBoost => (Color::srgb(1.0, 1.0, 0.0), Color::srgb(1.0, 1.0, 0.0)),
                PowerUpType::HealthPack => (Color::srgb(0.0, 1.0, 0.0), Color::srgb(0.0, 1.0, 0.0)),
                PowerUpType::EnergyRecharge => (Color::srgb(0.5, 0.0, 1.0), Color::srgb(0.5, 0.0, 1.0)),
                PowerUpType::TripleShot => (Color::srgb(1.0, 0.0, 1.0), Color::srgb(1.0, 0.0, 1.0)),
                PowerUpType::HomingMissiles => (Color::srgb(1.0, 0.0, 0.0), Color::srgb(1.0, 0.0, 0.0)),
            };
            
            // Spawn powerup entity
            let powerup_entity = commands.spawn((
                Transform::from_translation(position),
                Visibility::default(),
                PowerUp {
                    power_type,
                    lifetime: 30.0,
                    bob_phase: fastrand::f32() * std::f32::consts::TAU,
                },
                GameEntity,
            )).id();
            
            // Outer rotating container
            let container = commands.spawn((
                Mesh3d(meshes.add(Torus::new(2.0, 3.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive: emissive_color.into(),
                    metallic: 0.8,
                    perceptual_roughness: 0.2,
                    ..default()
                })),
                Transform::default(),
            )).id();
            
            // Inner glowing core
            let core = commands.spawn((
                Mesh3d(meshes.add(Sphere::new(1.5))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color.lighter(0.3),
                    emissive: (emissive_color.to_linear() * 2.0).into(),
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Transform::default(),
            )).id();
            
            // Build hierarchy
            commands.entity(powerup_entity).add_children(&[container]);
            commands.entity(container).add_children(&[core]);
        }
    }
}

pub fn animate_powerups(
    mut powerups: Query<(&mut Transform, &PowerUp)>,
    time: Res<Time>,
) {
    for (mut transform, powerup) in powerups.iter_mut() {
        let elapsed = time.elapsed_secs();
        
        // Rotation
        transform.rotate_y(time.delta_secs() * 2.0);
        
        // Bobbing motion
        let bob_amount = 2.0;
        let bob_speed = 1.5;
        let bob_offset = (elapsed * bob_speed + powerup.bob_phase).sin() * bob_amount;
        transform.translation.y += bob_offset * time.delta_secs();
        
        // Pulsing scale
        let pulse = (elapsed * 3.0).sin() * 0.1 + 1.0;
        transform.scale = Vec3::splat(pulse);
    }
}

pub fn collect_powerups_system(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut crate::enemies::Health, Entity), With<crate::Aircraft>>,
    powerups_query: Query<(Entity, &Transform, &PowerUp)>,
    mut active_powerups: ResMut<ActivePowerUps>,
    mut game_stats: ResMut<GameStats>,
    mut camera_query: Query<&mut FlightCamera>,
) {
    if let Ok((player_transform, mut player_health, player_entity)) = player_query.single_mut() {
        for (powerup_entity, powerup_transform, powerup) in powerups_query.iter() {
            let distance = player_transform.translation.distance(powerup_transform.translation);
            
            if distance < 8.0 {
                // Apply powerup effect
                match powerup.power_type {
                    PowerUpType::HealthPack => {
                        player_health.current = (player_health.current + 50.0).min(player_health.max);
                        game_stats.score += 50;
                    }
                    PowerUpType::EnergyRecharge => {
                        // Energy recharge handled in aircraft systems
                        game_stats.score += 50;
                    }
                    _ => {
                        // Duration-based powerups
                        let duration = match powerup.power_type {
                            PowerUpType::RapidFire => 10.0,
                            PowerUpType::Shield => 15.0,
                            PowerUpType::SpeedBoost => 8.0,
                            PowerUpType::TripleShot => 12.0,
                            PowerUpType::HomingMissiles => 20.0,
                            _ => 10.0,
                        };
                        
                        // Add effect component to player
                        commands.entity(player_entity).with_child((
                            PowerUpEffect {
                                effect_type: powerup.power_type,
                                duration,
                                remaining: duration,
                            },
                        ));
                        
                        // Update active powerups
                        match powerup.power_type {
                            PowerUpType::RapidFire => active_powerups.rapid_fire = true,
                            PowerUpType::Shield => active_powerups.shield = true,
                            PowerUpType::SpeedBoost => active_powerups.speed_multiplier = 2.0,
                            PowerUpType::TripleShot => active_powerups.triple_shot = true,
                            PowerUpType::HomingMissiles => active_powerups.homing_missiles = true,
                            _ => {}
                        }
                        
                        game_stats.score += 100;
                    }
                }
                
                // Remove powerup
                commands.entity(powerup_entity).despawn();
                
                // Camera effect
                if let Ok(mut camera) = camera_query.single_mut() {
                    camera.shake_amount = 1.0;
                    camera.shake_timer = 0.1;
                }
            }
        }
    }
}

pub fn update_powerup_effects(
    mut commands: Commands,
    mut effects_query: Query<(Entity, &mut PowerUpEffect)>,
    mut active_powerups: ResMut<ActivePowerUps>,
    time: Res<Time>,
) {
    for (effect_entity, mut effect) in effects_query.iter_mut() {
        effect.remaining -= time.delta_secs();
        
        if effect.remaining <= 0.0 {
            // Remove expired effect
            match effect.effect_type {
                PowerUpType::RapidFire => active_powerups.rapid_fire = false,
                PowerUpType::Shield => active_powerups.shield = false,
                PowerUpType::SpeedBoost => active_powerups.speed_multiplier = 1.0,
                PowerUpType::TripleShot => active_powerups.triple_shot = false,
                PowerUpType::HomingMissiles => active_powerups.homing_missiles = false,
                _ => {}
            }
            
            commands.entity(effect_entity).despawn();
        }
    }
}

pub fn cleanup_expired_powerups(
    mut commands: Commands,
    mut powerups_query: Query<(Entity, &mut PowerUp)>,
    time: Res<Time>,
) {
    for (entity, mut powerup) in powerups_query.iter_mut() {
        powerup.lifetime -= time.delta_secs();
        
        if powerup.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
pub struct ShieldVisual;

pub fn update_shield_visual(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<Entity, With<crate::Aircraft>>,
    shield_query: Query<Entity, With<ShieldVisual>>,
    active_powerups: Res<ActivePowerUps>,
    time: Res<Time>,
) {
    if let Ok(player_entity) = player_query.single() {
        let has_shield_visual = !shield_query.is_empty();
        
        if active_powerups.shield && !has_shield_visual {
            // Create shield visual
            let shield_entity = commands.spawn((
                Mesh3d(meshes.add(Sphere::new(8.0))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgba(0.0, 0.5, 1.0, 0.3),
                    emissive: Color::srgb(0.0, 0.3, 0.8).into(),
                    alpha_mode: AlphaMode::Blend,
                    double_sided: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::default(),
                ShieldVisual,
                crate::GameEntity,
            )).id();
            
            commands.entity(player_entity).add_child(shield_entity);
        } else if !active_powerups.shield && has_shield_visual {
            // Remove shield visual
            for shield_entity in shield_query.iter() {
                commands.entity(shield_entity).despawn();
            }
        }
        
        // Animate shield if active
        if active_powerups.shield {
            for shield_entity in shield_query.iter() {
                if let Ok(mut entity_commands) = commands.get_entity(shield_entity) {
                    // Pulsing effect
                    let pulse = (time.elapsed_secs() * 3.0).sin() * 0.05 + 0.95;
                    entity_commands.insert(Transform::from_scale(Vec3::splat(pulse)));
                }
            }
        }
    }
}