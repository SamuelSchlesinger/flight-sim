use bevy::prelude::*;
use std::collections::HashMap;
use crate::{Aircraft, GameEntity, game_state::GameStats};

#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
    pub damage: f32,
    pub attack_range: f32,
    pub pursuit_range: f32,
    pub enemy_type: EnemyType,
    pub behavior_state: EnemyBehaviorState,
    pub state_timer: f32,
    pub evasion_angle: f32,
    pub preferred_distance: f32,
    pub personality: PilotPersonality,
    pub formation_role: Option<FormationRole>,
    pub maneuver_skill: f32,
    pub reaction_time: f32,
    pub morale: f32,
    pub last_taunt_time: f32,
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

#[derive(Event)]
pub struct RadioChatterEvent {
    pub message: String,
    pub sender_type: EnemyType,
    pub personality: PilotPersonality,
}

#[derive(Debug, Clone, Copy)]
pub enum EnemyType {
    Fighter,
    Bomber,
    Ace,
}

#[derive(Debug, Clone, Copy)]
pub enum PilotPersonality {
    Aggressive,     // Rushes in, takes risks
    Defensive,      // Cautious, uses cover
    Tactical,       // Smart positioning, team player
    ShowOff,        // Does unnecessary maneuvers
    Veteran,        // Experienced, unpredictable
}

#[derive(Debug, Clone, Copy)]
pub enum FormationRole {
    Leader,
    Wingman(Entity), // Reference to leader
    Support,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnemyBehaviorState {
    Patrol,
    Pursuing,
    Attacking,
    Evading,
    Strafing,
    Retreating,
    Formation,
    Maneuvering(ManeuverType),
    Ambushing,
    Supporting,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ManeuverType {
    BarrelRoll,
    Immelmann,
    SplitS,
    HighYoYo,
    LowYoYo,
    Scissors,
    ChandelleTurn,
}

pub fn spawn_enemies_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<Aircraft>>,
    enemies: Query<Entity, With<Enemy>>,
    time: Res<Time>,
    mut spawn_timer: Local<f32>,
    game_stats: Res<crate::game_state::GameStats>,
) {
    let enemy_count = enemies.iter().count();
    let max_enemies = (5.0 * game_stats.difficulty_level).min(10.0) as usize;
    
    *spawn_timer += time.delta_secs();
    
    let spawn_interval = 3.0 / game_stats.difficulty_level.sqrt(); // Faster spawning at higher difficulty
    
    if enemy_count < max_enemies && *spawn_timer > spawn_interval {
        *spawn_timer = 0.0;
        
        if let Ok(player_transform) = player_query.single() {
            // Decide whether to spawn a formation or single enemy
            let spawn_formation = fastrand::f32() < 0.3 + (game_stats.difficulty_level - 1.0) * 0.1;
            
            if spawn_formation && enemy_count + 3 <= max_enemies {
                // Spawn a formation of 3 enemies
                spawn_enemy_formation(&mut commands, &mut meshes, &mut materials, player_transform, game_stats.difficulty_level);
            } else {
            // Spawn enemies at a distance from the player
            let spawn_distance = 150.0 + fastrand::f32() * 100.0;
            let angle = fastrand::f32() * std::f32::consts::TAU;
            let height = player_transform.translation.y + (-20.0 + fastrand::f32() * 40.0);
            
            let position = Vec3::new(
                player_transform.translation.x + angle.cos() * spawn_distance,
                height.max(30.0),
                player_transform.translation.z + angle.sin() * spawn_distance,
            );
            
            // Determine enemy type based on difficulty
            let ace_chance = 0.05 + (game_stats.difficulty_level - 1.0) * 0.1;
            let bomber_chance = 0.2 + (game_stats.difficulty_level - 1.0) * 0.1;
            
            let enemy_type = if fastrand::f32() < ace_chance {
                EnemyType::Ace
            } else if fastrand::f32() < bomber_chance {
                EnemyType::Bomber
            } else {
                EnemyType::Fighter
            };
            
            let (color, scale, speed, health, damage, preferred_distance) = match enemy_type {
                EnemyType::Fighter => (Color::srgb(0.8, 0.2, 0.2), 2.0, 60.0, 50.0, 10.0, 40.0),
                EnemyType::Bomber => (Color::srgb(0.4, 0.4, 0.4), 3.0, 40.0, 100.0, 20.0, 60.0),
                EnemyType::Ace => (Color::srgb(0.2, 0.2, 0.8), 1.8, 80.0, 75.0, 15.0, 30.0),
            };
            
            // Assign personality based on enemy type and randomness
            let personality = match enemy_type {
                EnemyType::Ace => {
                    if fastrand::f32() < 0.5 { PilotPersonality::Veteran } 
                    else { PilotPersonality::ShowOff }
                },
                EnemyType::Bomber => {
                    if fastrand::f32() < 0.7 { PilotPersonality::Defensive } 
                    else { PilotPersonality::Tactical }
                },
                EnemyType::Fighter => {
                    match fastrand::u32(0..4) {
                        0 => PilotPersonality::Aggressive,
                        1 => PilotPersonality::Defensive,
                        2 => PilotPersonality::Tactical,
                        _ => PilotPersonality::ShowOff,
                    }
                },
            };
            
            // Skill varies by type and personality
            let maneuver_skill = match (enemy_type, personality) {
                (EnemyType::Ace, _) => 0.8 + fastrand::f32() * 0.2,
                (_, PilotPersonality::Veteran) => 0.7 + fastrand::f32() * 0.2,
                (_, PilotPersonality::ShowOff) => 0.6 + fastrand::f32() * 0.3,
                (EnemyType::Fighter, _) => 0.4 + fastrand::f32() * 0.3,
                (EnemyType::Bomber, _) => 0.2 + fastrand::f32() * 0.2,
            };
            
            let reaction_time = match personality {
                PilotPersonality::Veteran => 0.2 + fastrand::f32() * 0.1,
                PilotPersonality::Aggressive => 0.3 + fastrand::f32() * 0.2,
                PilotPersonality::Defensive => 0.4 + fastrand::f32() * 0.2,
                _ => 0.5 + fastrand::f32() * 0.3,
            };
            
            // Spawn enemy aircraft
            let enemy_entity = commands.spawn((
                Transform::from_translation(position)
                    .looking_at(player_transform.translation, Vec3::Y),
                Visibility::default(),
                Enemy {
                    speed,
                    damage,
                    attack_range: 50.0,
                    pursuit_range: 200.0,
                    enemy_type,
                    behavior_state: EnemyBehaviorState::Patrol,
                    state_timer: 0.0,
                    evasion_angle: 0.0,
                    preferred_distance,
                    personality,
                    formation_role: None,
                    maneuver_skill,
                    reaction_time,
                    morale: 1.0,
                    last_taunt_time: 0.0,
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
}

fn spawn_enemy_formation(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    player_transform: &Transform,
    difficulty_level: f32,
) {
    // Formation spawn position
    let spawn_distance = 200.0 + fastrand::f32() * 100.0;
    let angle = fastrand::f32() * std::f32::consts::TAU;
    let height = player_transform.translation.y + (-10.0 + fastrand::f32() * 20.0);
    
    let formation_center = Vec3::new(
        player_transform.translation.x + angle.cos() * spawn_distance,
        height.max(40.0),
        player_transform.translation.z + angle.sin() * spawn_distance,
    );
    
    // Formation type and enemy type
    let use_fighters = fastrand::f32() < 0.7;
    let enemy_type = if use_fighters { EnemyType::Fighter } else { EnemyType::Bomber };
    
    // Spawn leader
    let leader_offset = Vec3::ZERO;
    let leader_entity = spawn_formation_enemy(
        commands, meshes, materials,
        formation_center + leader_offset,
        player_transform.translation,
        enemy_type,
        Some(FormationRole::Leader),
        difficulty_level,
    );
    
    // Spawn wingmen
    let wingman_offsets = [
        Vec3::new(-15.0, 0.0, -10.0),
        Vec3::new(15.0, 0.0, -10.0),
    ];
    
    for offset in &wingman_offsets {
        spawn_formation_enemy(
            commands, meshes, materials,
            formation_center + *offset,
            player_transform.translation,
            enemy_type,
            Some(FormationRole::Wingman(leader_entity)),
            difficulty_level,
        );
    }
}

fn spawn_formation_enemy(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    player_position: Vec3,
    enemy_type: EnemyType,
    formation_role: Option<FormationRole>,
    _difficulty_level: f32,
) -> Entity {
    let (color, scale, speed, health, damage, preferred_distance) = match enemy_type {
        EnemyType::Fighter => (Color::srgb(0.8, 0.2, 0.2), 2.0, 60.0, 50.0, 10.0, 40.0),
        EnemyType::Bomber => (Color::srgb(0.4, 0.4, 0.4), 3.0, 40.0, 100.0, 20.0, 60.0),
        EnemyType::Ace => (Color::srgb(0.2, 0.2, 0.8), 1.8, 80.0, 75.0, 15.0, 30.0),
    };
    
    // Formation members are more tactical
    let personality = match formation_role {
        Some(FormationRole::Leader) => PilotPersonality::Tactical,
        Some(FormationRole::Wingman(_)) => {
            if fastrand::f32() < 0.5 { PilotPersonality::Tactical } 
            else { PilotPersonality::Defensive }
        },
        _ => PilotPersonality::Tactical,
    };
    
    let maneuver_skill = 0.5 + fastrand::f32() * 0.3;
    let reaction_time = 0.3 + fastrand::f32() * 0.2;
    
    // Spawn enemy
    let enemy_entity = commands.spawn((
        Transform::from_translation(position)
            .looking_at(player_position, Vec3::Y),
        Visibility::default(),
        Enemy {
            speed,
            damage,
            attack_range: 50.0,
            pursuit_range: 200.0,
            enemy_type,
            behavior_state: EnemyBehaviorState::Formation,
            state_timer: 0.0,
            evasion_angle: 0.0,
            preferred_distance,
            personality,
            formation_role,
            maneuver_skill,
            reaction_time,
            morale: 1.2, // Formation bonus
            last_taunt_time: 0.0,
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
    
    // Enemy wings with formation markings
    let wing_color = if formation_role.is_some() {
        color.lighter(0.1) // Slightly different color for formation members
    } else {
        color.darker(0.2)
    };
    
    let left_wing = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(5.0 * scale, 0.2 * scale, 1.5 * scale))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: wing_color,
            metallic: 0.6,
            ..default()
        })),
        Transform::from_xyz(-3.0 * scale, 0.0, 0.0),
    )).id();
    
    let right_wing = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(5.0 * scale, 0.2 * scale, 1.5 * scale))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: wing_color,
            metallic: 0.6,
            ..default()
        })),
        Transform::from_xyz(3.0 * scale, 0.0, 0.0),
    )).id();
    
    // Build hierarchy
    commands.entity(enemy_entity).add_children(&[body]);
    commands.entity(body).add_children(&[left_wing, right_wing]);
    
    enemy_entity
}

pub fn enemy_ai_system(
    mut set: ParamSet<(
        Query<(Entity, &mut Transform, &mut Enemy, &Health), Without<Aircraft>>,
        Query<(Entity, &Transform, &Enemy), Without<Aircraft>>,
    )>,
    player_query: Query<(&Transform, &crate::Aircraft), With<Aircraft>>,
    time: Res<Time>,
    mut chatter_events: EventWriter<RadioChatterEvent>,
) {
    if let Ok((player_transform, player_aircraft)) = player_query.single() {
        let player_velocity = player_transform.forward() * player_aircraft.speed;
        
        // First collect formation data we need
        let mut formation_data = HashMap::<Entity, (Vec3, Quat)>::new();
        for (entity, transform, enemy) in set.p1().iter() {
            if matches!(enemy.formation_role, Some(FormationRole::Leader)) {
                formation_data.insert(entity, (transform.translation, transform.rotation));
            }
        }
        
        // Now update enemies
        for (entity, mut enemy_transform, mut enemy, health) in set.p0().iter_mut() {
            let to_player = player_transform.translation - enemy_transform.translation;
            let distance = to_player.length();
            
            // Update state timer and reaction
            enemy.state_timer -= time.delta_secs();
            
            // Update morale based on health
            enemy.morale = (health.current / health.max) * 1.2;
            if enemy.morale < 0.3 {
                enemy.behavior_state = EnemyBehaviorState::Retreating;
            }
            
            // Personality affects decision making
            let aggression_modifier = match enemy.personality {
                PilotPersonality::Aggressive => 1.5,
                PilotPersonality::Defensive => 0.7,
                PilotPersonality::ShowOff => 1.2,
                PilotPersonality::Veteran => 1.0,
                PilotPersonality::Tactical => 0.9,
            };
            
            // Radio chatter system
            if time.elapsed_secs() - enemy.last_taunt_time > 5.0 {
                if let Some(message) = generate_radio_chatter(&enemy, distance, health.current / health.max) {
                    chatter_events.write(RadioChatterEvent {
                        message,
                        sender_type: enemy.enemy_type,
                        personality: enemy.personality,
                    });
                    enemy.last_taunt_time = time.elapsed_secs();
                }
            }
            
            // State transitions
            match enemy.behavior_state {
                EnemyBehaviorState::Patrol => {
                    if distance < enemy.pursuit_range {
                        enemy.behavior_state = EnemyBehaviorState::Pursuing;
                        enemy.state_timer = 2.0;
                    } else {
                        // Patrol behavior - circle around spawn point
                        let patrol_angle = time.elapsed_secs() * 0.5;
                        let patrol_offset = Vec3::new(patrol_angle.cos() * 50.0, 0.0, patrol_angle.sin() * 50.0);
                        let patrol_target = enemy_transform.translation + patrol_offset;
                        
                        let to_patrol = (patrol_target - enemy_transform.translation).normalize();
                        let patrol_rotation = Transform::IDENTITY.looking_at(to_patrol, Vec3::Y).rotation;
                        enemy_transform.rotation = enemy_transform.rotation.slerp(patrol_rotation, time.delta_secs());
                        
                        let forward = enemy_transform.forward();
                        enemy_transform.translation += forward * enemy.speed * 0.5 * time.delta_secs();
                    }
                }
                
                EnemyBehaviorState::Pursuing => {
                    if distance < enemy.attack_range {
                        enemy.behavior_state = EnemyBehaviorState::Attacking;
                        enemy.state_timer = 1.5;
                    } else if distance > enemy.pursuit_range * 1.5 {
                        enemy.behavior_state = EnemyBehaviorState::Patrol;
                    } else {
                        // Advanced pursuit with prediction
                        let prediction_time = distance / enemy.speed;
                        let predicted_position = player_transform.translation + player_velocity * prediction_time * 0.5;
                        let to_predicted = (predicted_position - enemy_transform.translation).normalize();
                        
                        let target_rotation = Transform::IDENTITY.looking_at(to_predicted, Vec3::Y).rotation;
                        enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, time.delta_secs() * 3.0);
                        
                        let forward = enemy_transform.forward();
                        enemy_transform.translation += forward * enemy.speed * time.delta_secs();
                    }
                }
                
                EnemyBehaviorState::Attacking => {
                    if distance > enemy.attack_range * 1.2 {
                        enemy.behavior_state = EnemyBehaviorState::Pursuing;
                    } else if enemy.state_timer <= 0.0 {
                        // Personality-based attack pattern selection
                        let next_behavior = match enemy.personality {
                            PilotPersonality::Aggressive => {
                                if fastrand::f32() < 0.3 && enemy.maneuver_skill > 0.6 {
                                    EnemyBehaviorState::Maneuvering(ManeuverType::BarrelRoll)
                                } else {
                                    EnemyBehaviorState::Strafing
                                }
                            }
                            PilotPersonality::Defensive => {
                                if fastrand::f32() < 0.7 {
                                    EnemyBehaviorState::Evading
                                } else {
                                    EnemyBehaviorState::Strafing
                                }
                            }
                            PilotPersonality::ShowOff => {
                                if enemy.maneuver_skill > 0.5 {
                                    let maneuvers = [
                                        ManeuverType::BarrelRoll,
                                        ManeuverType::Immelmann,
                                        ManeuverType::ChandelleTurn,
                                    ];
                                    EnemyBehaviorState::Maneuvering(maneuvers[fastrand::usize(0..maneuvers.len())])
                                } else {
                                    EnemyBehaviorState::Strafing
                                }
                            }
                            PilotPersonality::Veteran => {
                                // Veterans make unpredictable choices
                                match fastrand::u32(0..4) {
                                    0 if enemy.maneuver_skill > 0.7 => {
                                        EnemyBehaviorState::Maneuvering(ManeuverType::HighYoYo)
                                    }
                                    1 => EnemyBehaviorState::Ambushing,
                                    2 => EnemyBehaviorState::Evading,
                                    _ => EnemyBehaviorState::Strafing,
                                }
                            }
                            PilotPersonality::Tactical => {
                                // For now, tactical enemies prefer strafing
                                // (checking for nearby allies would require additional queries)
                                EnemyBehaviorState::Strafing
                            }
                        };
                        
                        enemy.behavior_state = next_behavior;
                        enemy.evasion_angle = if fastrand::f32() > 0.5 { 1.0 } else { -1.0 };
                        enemy.state_timer = match next_behavior {
                            EnemyBehaviorState::Maneuvering(_) => 1.5,
                            _ => 2.0,
                        };
                    } else {
                        // Maintain optimal distance while attacking
                        let distance_error = distance - enemy.preferred_distance * aggression_modifier;
                        let forward = enemy_transform.forward();
                        
                        if distance_error.abs() > 5.0 {
                            let speed_multiplier = if distance_error > 0.0 { 1.0 } else { -0.5 };
                            enemy_transform.translation += forward * enemy.speed * speed_multiplier * time.delta_secs();
                        }
                        
                        // Keep facing player with personality-adjusted lead
                        let lead_factor = match (enemy.enemy_type, enemy.personality) {
                            (EnemyType::Ace, _) => 0.3 * enemy.maneuver_skill,
                            (_, PilotPersonality::Veteran) => 0.25 * enemy.maneuver_skill,
                            (_, PilotPersonality::Aggressive) => 0.15,
                            (EnemyType::Fighter, _) => 0.1,
                            (EnemyType::Bomber, _) => 0.0,
                        };
                        
                        let aim_point = player_transform.translation + player_velocity * lead_factor;
                        let to_aim = (aim_point - enemy_transform.translation).normalize();
                        let target_rotation = Transform::IDENTITY.looking_at(to_aim, Vec3::Y).rotation;
                        enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, time.delta_secs() * 2.5);
                    }
                }
                
                EnemyBehaviorState::Strafing => {
                    if enemy.state_timer <= 0.0 {
                        enemy.behavior_state = EnemyBehaviorState::Attacking;
                        enemy.state_timer = 1.0;
                    } else {
                        // Circle strafe around player
                        let radius = enemy.preferred_distance;
                        let strafe_speed = enemy.speed * 0.8;
                        let _angular_speed = strafe_speed / radius;
                        
                        // Calculate tangent direction
                        let to_player_normalized = to_player.normalize();
                        let right = to_player_normalized.cross(Vec3::Y).normalize() * enemy.evasion_angle;
                        
                        // Move in circular pattern
                        enemy_transform.translation += right * strafe_speed * time.delta_secs();
                        
                        // Face player while strafing
                        let target_rotation = Transform::IDENTITY.looking_at(to_player.normalize(), Vec3::Y).rotation;
                        enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, time.delta_secs() * 4.0);
                        
                        // Maintain altitude
                        enemy_transform.translation.y = player_transform.translation.y.clamp(20.0, 200.0);
                    }
                }
                
                EnemyBehaviorState::Evading => {
                    if enemy.state_timer <= 0.0 {
                        enemy.behavior_state = EnemyBehaviorState::Pursuing;
                    } else {
                        // Evasive maneuvers
                        let evasion_pattern = (time.elapsed_secs() * 3.0 + enemy.evasion_angle).sin();
                        let roll = evasion_pattern * 0.5;
                        let pitch = (time.elapsed_secs() * 2.0).cos() * 0.3;
                        
                        enemy_transform.rotate_local_x(pitch * time.delta_secs());
                        enemy_transform.rotate_local_z(roll * time.delta_secs());
                        
                        let forward = enemy_transform.forward();
                        enemy_transform.translation += forward * enemy.speed * 1.2 * time.delta_secs();
                    }
                }
                
                EnemyBehaviorState::Retreating => {
                    if distance > enemy.pursuit_range * 2.0 {
                        enemy.behavior_state = EnemyBehaviorState::Patrol;
                    } else {
                        // Flee from player
                        let away_from_player = -to_player.normalize();
                        let retreat_rotation = Transform::IDENTITY.looking_at(away_from_player, Vec3::Y).rotation;
                        enemy_transform.rotation = enemy_transform.rotation.slerp(retreat_rotation, time.delta_secs() * 2.0);
                        
                        let forward = enemy_transform.forward();
                        enemy_transform.translation += forward * enemy.speed * 0.8 * time.delta_secs();
                    }
                }
                
                EnemyBehaviorState::Formation => {
                    // Formation flying behavior
                    if let Some(formation_role) = &enemy.formation_role {
                        match formation_role {
                            FormationRole::Leader => {
                                // Leader behavior - similar to normal pursuit but maintains formation
                                if distance < enemy.attack_range {
                                    enemy.behavior_state = EnemyBehaviorState::Attacking;
                                    enemy.state_timer = 2.0;
                                } else if distance < enemy.pursuit_range {
                                    // Lead formation towards player
                                    let prediction_time = distance / enemy.speed;
                                    let predicted_position = player_transform.translation + player_velocity * prediction_time * 0.3;
                                    let to_predicted = (predicted_position - enemy_transform.translation).normalize();
                                    
                                    let target_rotation = Transform::IDENTITY.looking_at(to_predicted, Vec3::Y).rotation;
                                    enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, time.delta_secs() * 2.5);
                                    
                                    let forward = enemy_transform.forward();
                                    enemy_transform.translation += forward * enemy.speed * time.delta_secs();
                                }
                            }
                            FormationRole::Wingman(leader_entity) => {
                                // Find leader and maintain formation
                                if let Some((leader_position, leader_rotation)) = formation_data.get(leader_entity) {
                                    // Calculate desired position relative to leader
                                    let offset = if entity.index() % 2 == 0 {
                                        Vec3::new(-15.0, 0.0, -10.0)
                                    } else {
                                        Vec3::new(15.0, 0.0, -10.0)
                                    };
                                    
                                    let desired_position = *leader_position + 
                                        *leader_rotation * offset;
                                    
                                    let to_desired = desired_position - enemy_transform.translation;
                                    let formation_distance = to_desired.length();
                                    
                                    // Break formation if too far or in combat
                                    if formation_distance > 50.0 || distance < enemy.attack_range {
                                        enemy.behavior_state = EnemyBehaviorState::Pursuing;
                                    } else {
                                        // Maintain formation
                                        let target_rotation = *leader_rotation;
                                        enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, time.delta_secs() * 3.0);
                                        
                                        // Move to maintain position
                                        if formation_distance > 2.0 {
                                            let move_direction = to_desired.normalize();
                                            let speed_factor = (formation_distance / 20.0).min(1.5);
                                            enemy_transform.translation += move_direction * enemy.speed * speed_factor * time.delta_secs();
                                        }
                                    }
                                } else {
                                    // Leader is gone, act independently
                                    enemy.behavior_state = EnemyBehaviorState::Pursuing;
                                    enemy.formation_role = None;
                                }
                            }
                            FormationRole::Support => {
                                // Support role - provide covering fire
                                enemy.behavior_state = EnemyBehaviorState::Supporting;
                            }
                        }
                    }
                }
                
                EnemyBehaviorState::Maneuvering(maneuver_type) => {
                    // Execute advanced maneuvers
                    perform_maneuver(&mut enemy_transform, &mut enemy, maneuver_type, &time);
                    
                    if enemy.state_timer <= 0.0 {
                        // Return to appropriate state after maneuver
                        enemy.behavior_state = if distance < enemy.attack_range {
                            EnemyBehaviorState::Attacking
                        } else {
                            EnemyBehaviorState::Pursuing
                        };
                    }
                }
                
                EnemyBehaviorState::Ambushing => {
                    // Use terrain or altitude for ambush
                    if distance < enemy.attack_range * 1.5 {
                        // Spring the ambush
                        enemy.behavior_state = EnemyBehaviorState::Attacking;
                        enemy.state_timer = 2.0;
                    } else {
                        // Stay hidden, match player altitude from above
                        let ambush_height = player_transform.translation.y + 100.0;
                        enemy_transform.translation.y = enemy_transform.translation.y
                            .lerp(ambush_height, time.delta_secs() * 0.5);
                        
                        // Slowly move towards player's path
                        let future_position = player_transform.translation + player_velocity * 2.0;
                        let to_intercept = (future_position - enemy_transform.translation).normalize();
                        let intercept_rotation = Transform::IDENTITY.looking_at(to_intercept, Vec3::Y).rotation;
                        enemy_transform.rotation = enemy_transform.rotation.slerp(intercept_rotation, time.delta_secs());
                        
                        let forward = enemy_transform.forward();
                        enemy_transform.translation += forward * enemy.speed * 0.5 * time.delta_secs();
                    }
                }
                
                EnemyBehaviorState::Supporting => {
                    // Provide covering fire for allies
                    if distance < enemy.pursuit_range {
                        // Circle at medium range
                        let support_radius = enemy.preferred_distance * 1.5;
                        let angle = time.elapsed_secs() * 0.3;
                        
                        let target_position = player_transform.translation + Vec3::new(
                            angle.cos() * support_radius,
                            10.0,
                            angle.sin() * support_radius,
                        );
                        
                        let _to_target = (target_position - enemy_transform.translation).normalize();
                        let target_rotation = Transform::IDENTITY.looking_at(to_player.normalize(), Vec3::Y).rotation;
                        enemy_transform.rotation = enemy_transform.rotation.slerp(target_rotation, time.delta_secs() * 2.0);
                        
                        let forward = enemy_transform.forward();
                        enemy_transform.translation += forward * enemy.speed * 0.8 * time.delta_secs();
                    } else {
                        enemy.behavior_state = EnemyBehaviorState::Patrol;
                    }
                }
            }
            
            // Keep enemy within reasonable bounds
            enemy_transform.translation.y = enemy_transform.translation.y.clamp(10.0, 300.0);
            
            // Add slight wobble for more organic movement
            let wobble = (time.elapsed_secs() * 2.0 + enemy.evasion_angle).sin() * 0.02;
            enemy_transform.rotate_local_z(wobble);
        }
    }
}

fn perform_maneuver(
    transform: &mut Transform,
    enemy: &mut Enemy,
    maneuver_type: ManeuverType,
    time: &Time,
) {
    let skill_factor = enemy.maneuver_skill;
    let delta = time.delta_secs();
    
    match maneuver_type {
        ManeuverType::BarrelRoll => {
            // Full 360 degree roll
            let roll_speed = 3.0 * skill_factor;
            transform.rotate_local_z(roll_speed * delta);
            
            // Slight forward movement
            let forward = transform.forward();
            transform.translation += forward * enemy.speed * 0.7 * delta;
        }
        ManeuverType::Immelmann => {
            // Half loop followed by half roll
            let progress = 1.0 - (enemy.state_timer / 1.5);
            
            if progress < 0.5 {
                // Pull up phase
                transform.rotate_local_x(-2.0 * skill_factor * delta);
                let forward = transform.forward();
                transform.translation += forward * enemy.speed * 0.5 * delta;
            } else {
                // Roll phase
                transform.rotate_local_z(3.0 * skill_factor * delta);
                let forward = transform.forward();
                transform.translation += forward * enemy.speed * delta;
            }
        }
        ManeuverType::SplitS => {
            // Half roll followed by half loop (inverse Immelmann)
            let progress = 1.0 - (enemy.state_timer / 1.5);
            
            if progress < 0.3 {
                // Roll phase
                transform.rotate_local_z(3.0 * skill_factor * delta);
            } else {
                // Dive phase
                transform.rotate_local_x(2.0 * skill_factor * delta);
                let forward = transform.forward();
                transform.translation += forward * enemy.speed * 1.5 * delta;
            }
        }
        ManeuverType::HighYoYo => {
            // Climb and roll to get behind target
            let progress = 1.0 - (enemy.state_timer / 1.5);
            
            if progress < 0.4 {
                // Climb phase
                transform.rotate_local_x(-1.5 * skill_factor * delta);
                transform.rotate_local_z(0.5 * skill_factor * delta);
            } else {
                // Roll and dive phase
                transform.rotate_local_x(1.0 * skill_factor * delta);
                transform.rotate_local_z(2.0 * skill_factor * delta);
            }
            
            let forward = transform.forward();
            transform.translation += forward * enemy.speed * 0.8 * delta;
        }
        ManeuverType::LowYoYo => {
            // Dive and roll to gain speed
            let progress = 1.0 - (enemy.state_timer / 1.5);
            
            if progress < 0.4 {
                // Dive phase
                transform.rotate_local_x(1.5 * skill_factor * delta);
                transform.rotate_local_z(0.5 * skill_factor * delta);
            } else {
                // Pull up phase
                transform.rotate_local_x(-1.0 * skill_factor * delta);
            }
            
            let forward = transform.forward();
            let speed_boost = if progress < 0.4 { 1.3 } else { 0.9 };
            transform.translation += forward * enemy.speed * speed_boost * delta;
        }
        ManeuverType::Scissors => {
            // Weaving pattern
            let oscillation = (time.elapsed_secs() * 4.0).sin();
            transform.rotate_local_z(oscillation * 2.0 * skill_factor * delta);
            transform.rotate_local_x(oscillation * 0.5 * skill_factor * delta);
            
            let forward = transform.forward();
            transform.translation += forward * enemy.speed * 0.6 * delta;
        }
        ManeuverType::ChandelleTurn => {
            // Climbing turn
            transform.rotate_local_x(-0.5 * skill_factor * delta);
            transform.rotate_y(1.5 * skill_factor * delta);
            
            let forward = transform.forward();
            transform.translation += forward * enemy.speed * 0.7 * delta;
        }
    }
}

fn generate_radio_chatter(enemy: &Enemy, distance: f32, health_ratio: f32) -> Option<String> {
    // Don't spam chatter
    if fastrand::f32() > 0.3 {
        return None;
    }
    
    let message = match (enemy.behavior_state, enemy.personality) {
        (EnemyBehaviorState::Attacking, PilotPersonality::Aggressive) => {
            vec![
                "I've got you now!",
                "Nowhere to run!",
                "This is too easy!",
            ]
        }
        (EnemyBehaviorState::Attacking, PilotPersonality::ShowOff) => {
            vec![
                "Watch this move!",
                "Time for some aerobatics!",
                "Hope you're taking notes!",
            ]
        }
        (EnemyBehaviorState::Pursuing, PilotPersonality::Veteran) => {
            vec![
                "Target acquired.",
                "Beginning attack run.",
                "Stay focused, maintain pursuit.",
            ]
        }
        (EnemyBehaviorState::Evading, _) if health_ratio < 0.5 => {
            vec![
                "Taking heavy damage!",
                "I need backup!",
                "Systems failing!",
            ]
        }
        (EnemyBehaviorState::Formation, _) => {
            vec![
                "Formation holding.",
                "Following lead.",
                "Maintaining position.",
            ]
        }
        (EnemyBehaviorState::Retreating, _) => {
            vec![
                "Breaking off!",
                "I'm hit, pulling out!",
                "Can't take much more!",
            ]
        }
        (EnemyBehaviorState::Maneuvering(_), PilotPersonality::ShowOff) => {
            vec![
                "Check out this maneuver!",
                "Bet you can't do this!",
                "Textbook execution!",
            ]
        }
        _ if distance < 30.0 => {
            vec![
                "You're mine!",
                "Got a lock!",
                "In position!",
            ]
        }
        _ => return None,
    };
    
    if !message.is_empty() {
        Some(message[fastrand::usize(0..message.len())].to_string())
    } else {
        None
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
            
            // Check if enemy can shoot based on state and distance
            let can_attack = matches!(enemy.behavior_state, 
                EnemyBehaviorState::Attacking | 
                EnemyBehaviorState::Strafing | 
                EnemyBehaviorState::Supporting |
                EnemyBehaviorState::Formation
            ) && distance < enemy.attack_range;
                
            if can_attack {
                // Check if this specific enemy is on cooldown
                let can_shoot = !shoot_timers.contains_key(&enemy_entity);
                
                // Reaction time affects when they start shooting
                let reaction_delay_passed = enemy.state_timer < (2.0 - enemy.reaction_time);
                
                if can_shoot && reaction_delay_passed {
                    // Variable fire rate based on enemy type and personality
                    let fire_rate = match (enemy.enemy_type, enemy.personality) {
                        (EnemyType::Ace, _) => 0.8,
                        (_, PilotPersonality::Aggressive) => 0.9,
                        (EnemyType::Fighter, _) => 1.2,
                        (_, PilotPersonality::Defensive) => 1.8,
                        (EnemyType::Bomber, _) => 2.0,
                    };
                    
                    shoot_timers.insert(enemy_entity, fire_rate);
                    
                    // Calculate lead for better accuracy
                    let bullet_speed = 120.0;
                    let time_to_target = distance / bullet_speed;
                    let player_velocity = player_transform.forward() * 50.0; // Approximate player speed
                    let predicted_position = player_transform.translation + player_velocity * time_to_target * 0.5;
                    
                    // Accuracy varies by enemy type, personality, and skill
                    let base_accuracy = match enemy.enemy_type {
                        EnemyType::Ace => 0.05,
                        EnemyType::Fighter => 0.15,
                        EnemyType::Bomber => 0.25,
                    };
                    
                    let personality_modifier = match enemy.personality {
                        PilotPersonality::Veteran => 0.7,
                        PilotPersonality::Tactical => 0.8,
                        PilotPersonality::Aggressive => 1.2,
                        PilotPersonality::ShowOff => 1.5,
                        PilotPersonality::Defensive => 1.0,
                    };
                    
                    let accuracy_spread = base_accuracy * personality_modifier * (2.0 - enemy.maneuver_skill);
                    
                    let spread = Vec3::new(
                        (fastrand::f32() - 0.5) * accuracy_spread,
                        (fastrand::f32() - 0.5) * accuracy_spread,
                        (fastrand::f32() - 0.5) * accuracy_spread,
                    );
                    
                    let to_predicted = (predicted_position - enemy_transform.translation).normalize() + spread;
                    let bullet_velocity = to_predicted.normalize() * bullet_speed;
                    
                    // Spawn bullet with offset for visual appeal
                    let bullet_spawn = enemy_transform.translation + enemy_transform.forward() * 4.0;
                    
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
    active_powerups: Res<crate::powerups::ActivePowerUps>,
) {
    // Always update cooldown
    if *shoot_cooldown > 0.0 {
        *shoot_cooldown -= time.delta_secs();
    }
    
    // Adjust fire rate based on powerups
    let fire_rate = if active_powerups.rapid_fire { 0.1 } else { 0.25 };
    
    // Use Left Mouse Button or F key for shooting (not Space)
    if (*shoot_cooldown <= 0.0) && (keyboard.pressed(KeyCode::KeyF) || mouse.pressed(MouseButton::Left)) {
        if let Ok(player_transform) = player_query.single() {
            *shoot_cooldown = fire_rate;
            
            // Determine bullet pattern based on powerups
            let bullet_offsets = if active_powerups.triple_shot {
                vec![-3.0, 0.0, 3.0]
            } else {
                vec![-2.0, 2.0]
            };
            
            // Enhanced damage with powerups
            let damage = if active_powerups.homing_missiles { 50.0 } else { 25.0 };
            
            // Spawn bullets
            for offset in bullet_offsets {
                let bullet_spawn = player_transform.translation 
                    + player_transform.forward() * 5.0
                    + player_transform.right() * offset;
                
                let (color, emissive) = if active_powerups.homing_missiles {
                    (Color::srgb(1.0, 0.0, 0.5), Color::srgb(1.0, 0.0, 0.5))
                } else if active_powerups.triple_shot {
                    (Color::srgb(1.0, 0.0, 1.0), Color::srgb(1.0, 0.0, 1.0))
                } else {
                    (Color::srgb(0.0, 1.0, 1.0), Color::srgb(0.0, 1.0, 1.0))
                };
                
                commands.spawn((
                    Mesh3d(meshes.add(Sphere::new(0.3))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: color,
                        emissive: emissive.into(),
                        ..default()
                    })),
                    Transform::from_translation(bullet_spawn),
                    PlayerBullet {
                        velocity: player_transform.forward() * 250.0,
                        damage,
                        lifetime: 3.0,
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
    mut enemy_query: Query<(Entity, &Transform, &mut Health, &mut Enemy), Without<PlayerBullet>>,
    mut destroyed_events: EventWriter<EnemyDestroyedEvent>,
    mut game_stats: ResMut<GameStats>,
) {
    for (bullet_entity, bullet_transform, bullet) in bullet_query.iter() {
        for (enemy_entity, enemy_transform, mut health, mut enemy) in enemy_query.iter_mut() {
            let distance = bullet_transform.translation.distance(enemy_transform.translation);
            
            if distance < 5.0 {  // Increased hit box for easier targeting
                // Damage enemy
                health.current -= bullet.damage;
                
                // Remove bullet
                commands.entity(bullet_entity).despawn();
                
                // Check if enemy is destroyed
                if health.current <= 0.0 {
                    // Send destroyed event
                    destroyed_events.write(EnemyDestroyedEvent {
                        position: enemy_transform.translation,
                        enemy_type: enemy.enemy_type,
                    });
                    
                    // Award points
                    let points = match enemy.enemy_type {
                        EnemyType::Fighter => 50,
                        EnemyType::Bomber => 100,
                        EnemyType::Ace => 200,
                    };
                    game_stats.score += points;
                    game_stats.enemies_destroyed += 1;
                    
                    // Remove enemy
                    commands.entity(enemy_entity).despawn();
                } else if health.current < health.max * 0.3 && !matches!(enemy.behavior_state, EnemyBehaviorState::Retreating) {
                    // Low health - retreat
                    enemy.behavior_state = EnemyBehaviorState::Retreating;
                    enemy.state_timer = 5.0;
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
    active_powerups: Res<crate::powerups::ActivePowerUps>,
) {
    if let Ok((player_transform, mut player_health)) = player_query.single_mut() {
        for (bullet_entity, bullet_transform, bullet) in enemy_bullet_query.iter() {
            let distance = bullet_transform.translation.distance(player_transform.translation);
            
            if distance < 5.0 {  // Increased hit box for easier targeting
                // Check for shield
                if !active_powerups.shield {
                    // Damage player (no shield)
                    player_health.current -= bullet.damage;
                } else {
                    // Shield absorbs damage
                    player_health.current -= bullet.damage * 0.2; // 80% damage reduction
                }
                
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