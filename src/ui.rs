use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::game_state::{GameState, GameMode, CurrentGameMode, GameStats, ChallengeTimer, UpgradeData, get_upgrade_cost};

pub fn main_menu_ui(
    mut contexts: EguiContexts,
    mut game_state: ResMut<NextState<GameState>>,
    mut game_mode: ResMut<CurrentGameMode>,
    mut game_stats: ResMut<GameStats>,
    mut challenge_timer: ResMut<ChallengeTimer>,
) {
    let ctx = contexts.ctx_mut();
    
    egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(egui::RichText::new("✈️ SKY HUNTER").size(64.0).color(egui::Color32::from_rgb(255, 100, 100)));
                ui.add_space(20.0);
                
                ui.label(egui::RichText::new("Choose Your Challenge").size(24.0));
                ui.add_space(30.0);
                
                if ui.add_sized([300.0, 50.0], egui::Button::new(egui::RichText::new("🎯 Free Play").size(20.0))).clicked() {
                    game_mode.mode = GameMode::FreePlay;
                    game_stats.score = 0;
                    game_stats.combo = 0;
                    game_state.set(GameState::Playing);
                }
                ui.label("Fly freely and collect targets");
                
                ui.add_space(15.0);
                
                if ui.add_sized([300.0, 50.0], egui::Button::new(egui::RichText::new("⏱️ Time Attack").size(20.0))).clicked() {
                    game_mode.mode = GameMode::TimeAttack;
                    game_stats.score = 0;
                    game_stats.combo = 0;
                    challenge_timer.time_remaining = 60.0;
                    challenge_timer.total_time = 60.0;
                    game_state.set(GameState::Playing);
                }
                ui.label("Score as much as possible in 60 seconds");
                
                ui.add_space(15.0);
                
                if ui.add_sized([300.0, 50.0], egui::Button::new(egui::RichText::new("🎪 Target Hunt").size(20.0))).clicked() {
                    game_mode.mode = GameMode::TargetHunt;
                    game_stats.score = 0;
                    game_stats.combo = 0;
                    game_state.set(GameState::Playing);
                }
                ui.label("Find and destroy special targets");
                
                ui.add_space(15.0);
                
                if ui.add_sized([300.0, 50.0], egui::Button::new(egui::RichText::new("💀 Survival").size(20.0))).clicked() {
                    game_mode.mode = GameMode::Survival;
                    game_stats.score = 0;
                    game_stats.combo = 0;
                    challenge_timer.time_remaining = 10.0;
                    challenge_timer.total_time = 10.0;
                    game_state.set(GameState::Playing);
                }
                ui.label("Hit targets to gain time, miss and lose time");
                
                ui.add_space(15.0);
                
                if ui.add_sized([300.0, 50.0], egui::Button::new(egui::RichText::new("🏁 Race the Clock").size(20.0))).clicked() {
                    game_mode.mode = GameMode::RaceTheClock;
                    game_stats.score = 0;
                    game_stats.combo = 0;
                    challenge_timer.time_remaining = 120.0;
                    challenge_timer.total_time = 120.0;
                    game_state.set(GameState::Playing);
                }
                ui.label("Complete objectives before time runs out");
                
                ui.add_space(15.0);
                
                if ui.add_sized([300.0, 50.0], egui::Button::new(egui::RichText::new("🛠️ Upgrades").size(20.0))).clicked() {
                    game_state.set(GameState::UpgradeShop);
                }
                ui.label("Upgrade your aircraft");
                
                ui.add_space(40.0);
                
                // Stats
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("High Score: {}", game_stats.high_score)).size(16.0));
                    ui.add_space(40.0);
                    ui.label(egui::RichText::new(format!("💰 Coins: {}", game_stats.coins)).size(16.0));
                });
                
                ui.add_space(40.0);
                
                // Quit button
                if ui.add_sized([200.0, 40.0], egui::Button::new(egui::RichText::new("🚪 Quit Game").size(18.0))).clicked() {
                    std::process::exit(0);
                }
            });
        });
}

pub fn game_hud(
    mut contexts: EguiContexts,
    game_stats: Res<GameStats>,
    challenge_timer: Res<ChallengeTimer>,
    game_mode: Res<CurrentGameMode>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_query: Query<&crate::enemies::Health, With<crate::Aircraft>>,
    _active_powerups: Res<crate::powerups::ActivePowerUps>,
    powerup_effects: Query<&crate::powerups::PowerUpEffect>,
    mut radio_chatter_events: EventReader<crate::enemies::RadioChatterEvent>,
    mut chatter_display: Local<Vec<(String, f32, crate::enemies::EnemyType)>>,
    time: Res<Time>,
) {
    let ctx = contexts.ctx_mut();
    
    // Top panel for score and stats
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("Score: {}", game_stats.score)).size(24.0).color(egui::Color32::WHITE));
            ui.add_space(20.0);
            
            if game_stats.combo > 1 {
                ui.label(egui::RichText::new(format!("Combo x{}", game_stats.combo)).size(20.0).color(egui::Color32::YELLOW));
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                match game_mode.mode {
                    GameMode::TimeAttack | GameMode::Survival | GameMode::RaceTheClock => {
                        let time_color = if challenge_timer.time_remaining < 10.0 {
                            egui::Color32::RED
                        } else if challenge_timer.time_remaining < 30.0 {
                            egui::Color32::YELLOW
                        } else {
                            egui::Color32::WHITE
                        };
                        
                        ui.label(egui::RichText::new(format!("Time: {:.1}s", challenge_timer.time_remaining))
                            .size(24.0)
                            .color(time_color));
                    }
                    _ => {}
                }
                
                ui.add_space(20.0);
                ui.label(egui::RichText::new(format!("Targets Hit: {}", game_stats.targets_hit)).size(20.0));
            });
        });
    });
    
    // Mode-specific UI
    match game_mode.mode {
        GameMode::TargetHunt => {
            egui::TopBottomPanel::bottom("objective_panel").show(ctx, |ui| {
                ui.label(egui::RichText::new("🎯 Hunt for golden targets! They're worth 5x points!").size(18.0));
            });
        }
        GameMode::RaceTheClock => {
            egui::TopBottomPanel::bottom("objective_panel").show(ctx, |ui| {
                ui.label(egui::RichText::new("🏁 Hit 50 targets before time runs out!").size(18.0));
            });
        }
        _ => {}
    }
    
    // Speed indicator
    egui::Area::new(egui::Id::new("speed_indicator"))
        .anchor(egui::Align2::LEFT_TOP, [10.0, 50.0])
        .show(ctx, |ui| {
            let speed_text = if keyboard_input.pressed(KeyCode::Space) {
                "BOOST!"
            } else if keyboard_input.pressed(KeyCode::KeyW) {
                "Fast"
            } else if keyboard_input.pressed(KeyCode::KeyS) {
                "Slow"
            } else {
                "Normal"
            };
            
            let speed_color = if keyboard_input.pressed(KeyCode::Space) {
                egui::Color32::from_rgb(255, 150, 0)
            } else if keyboard_input.pressed(KeyCode::KeyW) {
                egui::Color32::from_rgb(0, 255, 0)
            } else if keyboard_input.pressed(KeyCode::KeyS) {
                egui::Color32::from_rgb(255, 255, 0)
            } else {
                egui::Color32::WHITE
            };
            
            ui.label(egui::RichText::new(format!("Speed: {speed_text}")).size(18.0).color(speed_color));
        });
    
    // Health bar
    if let Ok(health) = player_query.single() {
        egui::Area::new(egui::Id::new("health_bar"))
            .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -40.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("HEALTH").size(16.0).color(egui::Color32::WHITE));
                    ui.add_space(10.0);
                    
                    let health_percentage = health.current / health.max;
                    let health_color = if health_percentage > 0.6 {
                        egui::Color32::GREEN
                    } else if health_percentage > 0.3 {
                        egui::Color32::YELLOW
                    } else {
                        egui::Color32::RED
                    };
                    
                    ui.add(egui::ProgressBar::new(health_percentage)
                        .desired_width(200.0)
                        .fill(health_color)
                        .text(format!("{:.0}/{:.0}", health.current, health.max)));
                });
            });
    }
    
    // Controls hint
    egui::Area::new(egui::Id::new("controls_hint"))
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("CONTROLS").size(16.0).color(egui::Color32::WHITE));
                ui.label(egui::RichText::new("Mouse - Look/Turn").size(14.0).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new("A/D - Roll").size(14.0).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new("W/S - Speed Up/Down").size(14.0).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new("Space - Boost").size(14.0).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new("Left Click/F - Shoot").size(14.0).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new("ESC - Pause").size(14.0).color(egui::Color32::GRAY));
            });
        });
    
    // Active powerups display
    egui::Area::new(egui::Id::new("powerups_display"))
        .anchor(egui::Align2::RIGHT_TOP, [-10.0, 50.0])
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                if !powerup_effects.is_empty() {
                    ui.label(egui::RichText::new("ACTIVE POWERUPS").size(16.0).color(egui::Color32::WHITE));
                    ui.add_space(5.0);
                    
                    for effect in powerup_effects.iter() {
                        let (icon, name, color) = match effect.effect_type {
                            crate::powerups::PowerUpType::RapidFire => ("🔥", "Rapid Fire", egui::Color32::from_rgb(255, 128, 0)),
                            crate::powerups::PowerUpType::Shield => ("🛡️", "Shield", egui::Color32::from_rgb(0, 128, 255)),
                            crate::powerups::PowerUpType::SpeedBoost => ("⚡", "Speed Boost", egui::Color32::from_rgb(255, 255, 0)),
                            crate::powerups::PowerUpType::TripleShot => ("🎯", "Triple Shot", egui::Color32::from_rgb(255, 0, 255)),
                            crate::powerups::PowerUpType::HomingMissiles => ("🚀", "Homing", egui::Color32::from_rgb(255, 0, 0)),
                            _ => continue,
                        };
                        
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("{icon} {name}")).size(14.0).color(color));
                            ui.label(egui::RichText::new(format!("{:.1}s", effect.remaining)).size(12.0).color(egui::Color32::GRAY));
                        });
                    }
                }
            });
        });
    
    // Process new radio chatter events
    for event in radio_chatter_events.read() {
        chatter_display.push((event.message.clone(), 5.0, event.sender_type));
        
        // Keep only the last 5 messages
        if chatter_display.len() > 5 {
            chatter_display.remove(0);
        }
    }
    
    // Update and display radio chatter
    chatter_display.retain_mut(|(_, timer, _)| {
        *timer -= time.delta_secs();
        *timer > 0.0
    });
    
    if !chatter_display.is_empty() {
        egui::Area::new(egui::Id::new("radio_chatter"))
            .anchor(egui::Align2::LEFT_TOP, [10.0, 80.0])
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("RADIO CHATTER").size(14.0).color(egui::Color32::LIGHT_GRAY));
                    ui.add_space(5.0);
                    
                    for (message, timer, enemy_type) in chatter_display.iter() {
                        let alpha = (*timer / 5.0 * 255.0) as u8;
                        let color = match enemy_type {
                            crate::enemies::EnemyType::Fighter => egui::Color32::from_rgba_unmultiplied(255, 100, 100, alpha),
                            crate::enemies::EnemyType::Bomber => egui::Color32::from_rgba_unmultiplied(150, 150, 150, alpha),
                            crate::enemies::EnemyType::Ace => egui::Color32::from_rgba_unmultiplied(100, 100, 255, alpha),
                        };
                        
                        ui.label(egui::RichText::new(format!("📻 {}", message)).size(12.0).color(color));
                    }
                });
            });
    }
    
    // Pause handling moved to handle_escape_key in main.rs
}

pub fn pause_menu(
    mut contexts: EguiContexts,
    mut game_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_stats: ResMut<GameStats>,
    mut challenge_timer: ResMut<ChallengeTimer>,
    game_mode: Res<CurrentGameMode>,
) {
    let ctx = contexts.ctx_mut();
    
    // Dark overlay
    egui::Area::new(egui::Id::new("pause_overlay"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(-10000.0, -10000.0),
                    egui::vec2(20000.0, 20000.0)
                ),
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180)
            );
        });
    
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(150.0);
            
            ui.heading(egui::RichText::new("⏸️ GAME PAUSED").size(56.0).color(egui::Color32::WHITE));
            ui.add_space(20.0);
            
            ui.label(egui::RichText::new(format!("Current Score: {}", game_stats.score)).size(24.0).color(egui::Color32::LIGHT_GRAY));
            ui.add_space(40.0);
            
            if ui.add_sized([250.0, 60.0], egui::Button::new(egui::RichText::new("▶️ Resume").size(24.0))).clicked() 
                || keyboard_input.just_pressed(KeyCode::Escape) {
                game_state.set(GameState::Playing);
            }
            ui.label(egui::RichText::new("Press ESC to resume").size(14.0).color(egui::Color32::GRAY));
            
            ui.add_space(20.0);
            
            if ui.add_sized([250.0, 60.0], egui::Button::new(egui::RichText::new("🔄 Restart").size(24.0))).clicked() {
                // Reset game stats for restart
                game_stats.score = 0;
                game_stats.combo = 0;
                game_stats.targets_hit = 0;
                game_stats.time_played = 0.0;
                
                // Reset timer based on game mode
                match game_mode.mode {
                    GameMode::TimeAttack => {
                        challenge_timer.time_remaining = 60.0;
                        challenge_timer.total_time = 60.0;
                    }
                    GameMode::Survival => {
                        challenge_timer.time_remaining = 10.0;
                        challenge_timer.total_time = 10.0;
                    }
                    GameMode::RaceTheClock => {
                        challenge_timer.time_remaining = 120.0;
                        challenge_timer.total_time = 120.0;
                    }
                    _ => {}
                }
                
                game_state.set(GameState::Playing);
            }
            
            ui.add_space(20.0);
            
            if ui.add_sized([250.0, 60.0], egui::Button::new(egui::RichText::new("🏠 Main Menu").size(24.0))).clicked() {
                game_state.set(GameState::MainMenu);
            }
            
            ui.add_space(20.0);
            
            if ui.add_sized([250.0, 60.0], egui::Button::new(egui::RichText::new("🚪 Quit Game").size(24.0))).clicked() {
                std::process::exit(0);
            }
            
            ui.add_space(40.0);
            ui.separator();
            ui.add_space(20.0);
            
            ui.label(egui::RichText::new("⚙️ SETTINGS").size(20.0).color(egui::Color32::WHITE));
            ui.add_space(10.0);
            ui.label(egui::RichText::new("Press F11 to toggle fullscreen").size(16.0).color(egui::Color32::LIGHT_GRAY));
        });
    });
}

pub fn game_over_screen(
    mut contexts: EguiContexts,
    mut game_state: ResMut<NextState<GameState>>,
    mut game_stats: ResMut<GameStats>,
    mut challenge_timer: ResMut<ChallengeTimer>,
    game_mode: Res<CurrentGameMode>,
) {
    let ctx = contexts.ctx_mut();
    
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(150.0);
            
            ui.heading(egui::RichText::new("GAME OVER").size(48.0).color(egui::Color32::RED));
            ui.add_space(30.0);
            
            ui.label(egui::RichText::new(format!("Final Score: {}", game_stats.score)).size(32.0));
            ui.label(egui::RichText::new(format!("Targets Hit: {}", game_stats.targets_hit)).size(24.0));
            ui.label(egui::RichText::new(format!("Max Combo: {}", game_stats.max_combo)).size(24.0));
            
            ui.add_space(20.0);
            
            let coins_earned = game_stats.score / 100;
            ui.label(egui::RichText::new(format!("💰 Coins Earned: {coins_earned}")).size(20.0).color(egui::Color32::YELLOW));
            
            ui.add_space(40.0);
            
            if ui.add_sized([200.0, 50.0], egui::Button::new(egui::RichText::new("Play Again").size(20.0))).clicked() {
                // Reset game stats for play again
                game_stats.score = 0;
                game_stats.combo = 0;
                game_stats.targets_hit = 0;
                game_stats.time_played = 0.0;
                
                // Reset timer based on game mode
                match game_mode.mode {
                    GameMode::TimeAttack => {
                        challenge_timer.time_remaining = 60.0;
                        challenge_timer.total_time = 60.0;
                    }
                    GameMode::Survival => {
                        challenge_timer.time_remaining = 10.0;
                        challenge_timer.total_time = 10.0;
                    }
                    GameMode::RaceTheClock => {
                        challenge_timer.time_remaining = 120.0;
                        challenge_timer.total_time = 120.0;
                    }
                    _ => {}
                }
                
                game_state.set(GameState::Playing);
            }
            
            ui.add_space(10.0);
            
            if ui.add_sized([200.0, 50.0], egui::Button::new(egui::RichText::new("Main Menu").size(20.0))).clicked() {
                game_state.set(GameState::MainMenu);
            }
        });
    });
}

pub fn upgrade_shop_ui(
    mut contexts: EguiContexts,
    mut game_state: ResMut<NextState<GameState>>,
    mut game_stats: ResMut<GameStats>,
    mut upgrades: ResMut<UpgradeData>,
) {
    let ctx = contexts.ctx_mut();
    
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            
            ui.heading(egui::RichText::new("🛠️ UPGRADE SHOP").size(48.0));
            ui.label(egui::RichText::new(format!("💰 Coins: {}", game_stats.coins)).size(24.0).color(egui::Color32::YELLOW));
            ui.add_space(40.0);
            
            // Speed upgrade
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 250.0);
                ui.label(egui::RichText::new("🚀 Speed").size(20.0));
                ui.add_space(20.0);
                ui.label(format!("Level {}", upgrades.speed_level));
                ui.add_space(20.0);
                
                let cost = get_upgrade_cost(upgrades.speed_level);
                let can_afford = game_stats.coins >= cost;
                
                if ui.add_enabled(can_afford, egui::Button::new(format!("Upgrade (💰 {cost})"))).clicked() {
                    game_stats.coins -= cost;
                    upgrades.speed_level += 1;
                }
            });
            
            ui.add_space(20.0);
            
            // Maneuverability upgrade
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 250.0);
                ui.label(egui::RichText::new("🎯 Maneuverability").size(20.0));
                ui.add_space(20.0);
                ui.label(format!("Level {}", upgrades.maneuverability_level));
                ui.add_space(20.0);
                
                let cost = get_upgrade_cost(upgrades.maneuverability_level);
                let can_afford = game_stats.coins >= cost;
                
                if ui.add_enabled(can_afford, egui::Button::new(format!("Upgrade (💰 {cost})"))).clicked() {
                    game_stats.coins -= cost;
                    upgrades.maneuverability_level += 1;
                }
            });
            
            ui.add_space(20.0);
            
            // Magnet upgrade
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 250.0);
                ui.label(egui::RichText::new("🧲 Target Magnet").size(20.0));
                ui.add_space(20.0);
                ui.label(format!("Level {}", upgrades.magnet_level));
                ui.add_space(20.0);
                
                let cost = get_upgrade_cost(upgrades.magnet_level + 1);
                let can_afford = game_stats.coins >= cost;
                
                if ui.add_enabled(can_afford, egui::Button::new(format!("Upgrade (💰 {cost})"))).clicked() {
                    game_stats.coins -= cost;
                    upgrades.magnet_level += 1;
                }
            });
            
            ui.add_space(20.0);
            
            // Score multiplier upgrade
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 250.0);
                ui.label(egui::RichText::new("💎 Score Multiplier").size(20.0));
                ui.add_space(20.0);
                ui.label(format!("Level {}", upgrades.multiplier_level));
                ui.add_space(20.0);
                
                let cost = get_upgrade_cost(upgrades.multiplier_level);
                let can_afford = game_stats.coins >= cost;
                
                if ui.add_enabled(can_afford, egui::Button::new(format!("Upgrade (💰 {cost})"))).clicked() {
                    game_stats.coins -= cost;
                    upgrades.multiplier_level += 1;
                }
            });
            
            ui.add_space(60.0);
            
            if ui.add_sized([200.0, 50.0], egui::Button::new(egui::RichText::new("Back").size(20.0))).clicked() {
                game_state.set(GameState::MainMenu);
            }
        });
    });
}