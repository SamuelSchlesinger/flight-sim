use bevy::prelude::*;
use std::collections::HashMap;
use crate::{Aircraft, GameEntity, game_state::GameStats};

#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
    pub health: f32,
    pub damage: f32,
    pub attack_range: f32,
    pub pursuit_range: f32,
}

#[derive(Component)]
pub struct EnemyBullet {
    pub velocity: Vec3,
    pub damage: f32,
    pub lifetime: f32,
}

#[derive(Component)]
pub struct PlayerBullet {
    pub velocity: Vec3,
    pub damage: f32,
    pub lifetime: f32,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Event)]
pub struct EnemyDestroyedEvent {
    pub position: Vec3,
    pub enemy_type: EnemyType,
}

#[derive(Debug, Clone, Copy)]
pub enum EnemyType {
    Fighter,
    Bomber,
    Ace,
}

pub fn spawn_enemies_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<Aircraft>>,
    enemies: Query<Entity, With<Enemy>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
) {
    let enemy_count = enemies.iter().count();
    let max_enemies = 5;
    
    *spawn_timer += time.delta_secs();
    
    if enemy_count < max_enemies && *spawn_timer > 3.0 {
        *spawn_timer = 0.0;
        
        if let Ok(player_transform) = player_query.single() {
            // Spawn enemies at a distance from the player
            let spawn_distance = 150.0 + fastrand::f32() * 100.0;
            let angle = fastrand::f32() * std::f32::consts::TAU;
            let height = player_transform.translation.y + (-20.0 + fastrand::f32() * 40.0);
            
            let position = Vec3::new(
                player_transform.translation.x + angle.cos() * spawn_distance,
                height.max(30.0),
                player_transform.translation.z + angle.sin() * spawn_distance,
            );
            
            // Determine enemy type
            let enemy_type = if fastrand::f32() < 0.1 {
                EnemyType::Ace
            } else if fastrand::f32() < 0.3 {
                EnemyType::Bomber
            } else {
                EnemyType::Fighter
            };
            
            let (color, scale, speed, health, damage) = match enemy_type {
                EnemyType::Fighter => (Color::srgb(0.8, 0.2, 0.2), 2.0, 60.0, 50.0, 10.0),
                EnemyType::Bomber => (Color::srgb(0.4, 0.4, 0.4), 3.0, 40.0, 100.0, 20.0),
                EnemyType::Ace => (Color::srgb(0.2, 0.2, 0.8), 1.8, 80.0, 75.0, 15.0),
            };
            
            // Spawn enemy aircraft
            let enemy_entity = commands.spawn((
                Transform::from_translation(position)
                    .looking_at(player_transform.translation, Vec3::Y),
                Visibility::default(),
                Enemy {
                    speed,
                    health,
                    damage,
                    attack_range: 50.0,
                    pursuit_range: 200.0,
                },
                Health {
                    current: health,
                    max: health,
                },
                GameEntity,
            )).id();
            
            // Enemy body
            let body = commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(1.2 * scale, 0.8 * scale, 3.0 * scale))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    metallic: 0.7,
                    perceptual_roughness: 0.3,
                    ..default()
                })),
                Transform::default(),
            )).id();
            
            // Enemy wings
            let left_wing = commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(5.0 * scale, 0.2 * scale, 1.5 * scale))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color.darker(0.2),
                    metallic: 0.6,
                    ..default()
                })),
                Transform::from_xyz(-3.0 * scale, 0.0, 0.0),
            )).id();
            
            let right_wing = commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(5.0 * scale, 0.2 * scale, 1.5 * scale))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color.darker(0.2),
                    metallic: 0.6,
                    ..default()
                })),
                Transform::from_xyz(3.0 * scale, 0.0, 0.0),
            )).id();
            
            // Build hierarchy
            commands.entity(enemy_entity).add_children(&[body]);
            commands.entity(body).add_children(&[left_wing, right_wing]);
        }
    }
}

pub fn enemy_ai_system(
    mut enemy_query: Query<(&mut Transform, &Enemy), Without<Aircraft>>,
    player_query: Query<&Transform, With<Aircraft>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut enemy_transform, enemy) in enemy_query.iter_mut() {
            let to_player = player_transform.translation - enemy_transform.translation;
            let distance = to_player.length();
            
            if distance < enemy.pursuit_range {
                // Look at player
                let look_direction = to_player.normalize();
                let target_rotation = Transform::IDENTITY
                    .looking_at(look_direction, Vec3::Y)
                    .rotation;
                
                // Smooth rotation
                enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, time.delta_secs() * 2.0);
                
                // Move towards player
                if distance > enemy.attack_range {
                    let forward = enemy_transform.forward();
                    enemy_transform.translation += forward * enemy.speed * time.delta_secs();
                }
                
                // Keep enemy above ground
                if enemy_transform.translation.y < 20.0 {
                    enemy_transform.translation.y = 20.0;
                }
            }
        }
    }
}

pub fn enemy_shooting_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    enemy_query: Query<(Entity, &Transform, &Enemy)>,
    player_query: Query<&Transform, With<Aircraft>>,
    time: Res<Time>,
    mut shoot_timers: Local<HashMap<Entity, f32>>,
) {
    if let Ok(player_transform) = player_query.single() {
        // Update all timers
        for timer in shoot_timers.values_mut() {
            *timer -= time.delta_secs();
        }
        shoot_timers.retain(|_, timer| *timer > 0.0);
        
        for (enemy_entity, enemy_transform, enemy) in enemy_query.iter() {
            let to_player = player_transform.translation - enemy_transform.translation;
            let distance = to_player.length();
            
            // Check if enemy can shoot
            if distance < enemy.attack_range {
                // Check if this specific enemy is on cooldown
                let can_shoot = !shoot_timers.contains_key(&enemy_entity);
                
                if can_shoot {
                    // Add enemy-specific cooldown
                    shoot_timers.insert(enemy_entity, 1.5); // Enemy fire rate
                    
                    // Spawn bullet
                    let bullet_spawn = enemy_transform.translation + enemy_transform.forward() * 3.0;
                    let bullet_velocity = to_player.normalize() * 100.0;
                    
                    commands.spawn((
                        Mesh3d(meshes.add(Sphere::new(0.3))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(1.0, 0.0, 0.0),
                            emissive: Color::srgb(1.0, 0.0, 0.0).into(),
                            ..default()
                        })),
                        Transform::from_translation(bullet_spawn),
                        EnemyBullet {
                            velocity: bullet_velocity,
                            damage: enemy.damage,
                            lifetime: 3.0,
                        },
                        GameEntity,
                    ));
                }
            }
        }
    }
}

pub fn player_shooting_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<Aircraft>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut shoot_cooldown: Local<f32>,
) {
    // Always update cooldown
    if *shoot_cooldown > 0.0 {
        *shoot_cooldown -= time.delta_secs();
    }
    
    // Use Left Mouse Button or F key for shooting (not Space)
    if (*shoot_cooldown <= 0.0) && (keyboard.pressed(KeyCode::KeyF) || mouse.pressed(MouseButton::Left)) {
        if let Ok(player_transform) = player_query.single() {
            *shoot_cooldown = 0.25; // Slower, consistent fire rate
            
            // Spawn two bullets (one from each wing)
            for offset in [-2.0, 2.0] {
                let bullet_spawn = player_transform.translation 
                    + player_transform.forward() * 5.0
                    + player_transform.right() * offset;
                
                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.2))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: Color::srgb(0.0, 1.0, 1.0),
                        emissive: Color::srgb(0.0, 1.0, 1.0).into(),
                        ..default()
                    })),
                    Transform::from_translation(bullet_spawn),
                    PlayerBullet {
                        velocity: player_transform.forward() * 200.0,
                        damage: 25.0,
                        lifetime: 2.0,
                    },
                    GameEntity,
                ));
            }
        }
    }
}

pub fn update_bullets_system(
    mut commands: Commands,
    mut bullet_query: Query<(Entity, &mut Transform, &mut PlayerBullet), Without<EnemyBullet>>,
    mut enemy_bullet_query: Query<(Entity, &mut Transform, &mut EnemyBullet)>,
    time: Res<Time>,
) {
    // Update player bullets
    for (entity, mut transform, mut bullet) in bullet_query.iter_mut() {
        transform.translation += bullet.velocity * time.delta_secs();
        bullet.lifetime -= time.delta_secs();
        
        if bullet.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
    
    // Update enemy bullets
    for (entity, mut transform, mut bullet) in enemy_bullet_query.iter_mut() {
        transform.translation += bullet.velocity * time.delta_secs();
        bullet.lifetime -= time.delta_secs();
        
        if bullet.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn bullet_collision_system(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform, &PlayerBullet)>,
    mut enemy_query: Query<(Entity, &Transform, &mut Health, &Enemy), Without<PlayerBullet>>,
    mut destroyed_events: EventWriter<EnemyDestroyedEvent>,
    mut game_stats: ResMut<GameStats>,
) {
    for (bullet_entity, bullet_transform, bullet) in bullet_query.iter() {
        for (enemy_entity, enemy_transform, mut health, enemy) in enemy_query.iter_mut() {
            let distance = bullet_transform.translation.distance(enemy_transform.translation);
            
            if distance < 5.0 {  // Increased hit box for easier targeting
                // Damage enemy
                health.current -= bullet.damage;
                
                // Remove bullet
                commands.entity(bullet_entity).despawn();
                
                // Check if enemy is destroyed
                if health.current <= 0.0 {
                    // Determine enemy type for event
                    let enemy_type = if enemy.speed > 70.0 {
                        EnemyType::Ace
                    } else if enemy.health > 80.0 {
                        EnemyType::Bomber
                    } else {
                        EnemyType::Fighter
                    };
                    
                    // Send destroyed event
                    destroyed_events.write(EnemyDestroyedEvent {
                        position: enemy_transform.translation,
                        enemy_type,
                    });
                    
                    // Award points
                    let points = match enemy_type {
                        EnemyType::Fighter => 50,
                        EnemyType::Bomber => 100,
                        EnemyType::Ace => 200,
                    };
                    game_stats.score += points;
                    
                    // Remove enemy
                    commands.entity(enemy_entity).despawn();
                }
                
                break;
            }
        }
    }
}

pub fn player_damage_system(
    mut commands: Commands,
    enemy_bullet_query: Query<(Entity, &Transform, &EnemyBullet)>,
    mut player_query: Query<(&Transform, &mut Health), With<Aircraft>>,
    mut game_state: ResMut<NextState<crate::game_state::GameState>>,
) {
    if let Ok((player_transform, mut player_health)) = player_query.single_mut() {
        for (bullet_entity, bullet_transform, bullet) in enemy_bullet_query.iter() {
            let distance = bullet_transform.translation.distance(player_transform.translation);
            
            if distance < 5.0 {  // Increased hit box for easier targeting
                // Damage player
                player_health.current -= bullet.damage;
                
                // Remove bullet
                commands.entity(bullet_entity).despawn();
                
                // Check if player is destroyed
                if player_health.current <= 0.0 {
                    game_state.set(crate::game_state::GameState::GameOver);
                }
            }
        }
    }
}

pub fn spawn_explosion_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut destroyed_events: EventReader<EnemyDestroyedEvent>,
) {
    for event in destroyed_events.read() {
        // Spawn explosion particles
        for _ in 0..20 {
            let velocity = Vec3::new(
                (fastrand::f32() - 0.5) * 30.0,
                fastrand::f32() * 20.0,
                (fastrand::f32() - 0.5) * 30.0,
            );
            
            let color = match event.enemy_type {
                EnemyType::Fighter => Color::srgb(1.0, 0.5, 0.0),
                EnemyType::Bomber => Color::srgb(0.8, 0.8, 0.0),
                EnemyType::Ace => Color::srgb(0.5, 0.5, 1.0),
            };
            
            commands.spawn((
                Mesh3d(meshes.add(Sphere::new(0.5))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive: color.into(),
                    ..default()
                })),
                Transform::from_translation(event.position),
                crate::targets::ParticleEffect {
                    lifetime: 1.0,
                    velocity,
                },
                GameEntity,
            ));
        }
    }
}