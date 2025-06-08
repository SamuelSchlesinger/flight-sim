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
            });
        });
}

pub fn game_hud(
    mut contexts: EguiContexts,
    game_stats: Res<GameStats>,
    challenge_timer: Res<ChallengeTimer>,
    game_mode: Res<CurrentGameMode>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
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
    
    // Controls hint
    egui::Area::new(egui::Id::new("controls_hint"))
        .anchor(egui::Align2::RIGHT_BOTTOM, [-10.0, -10.0])
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("ESC - Pause").size(14.0).color(egui::Color32::GRAY));
        });
    
    // Check for pause
    if keyboard_input.just_pressed(KeyCode::Escape) {
        game_state.set(GameState::Paused);
    }
}

pub fn pause_menu(
    mut contexts: EguiContexts,
    mut game_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let ctx = contexts.ctx_mut();
    
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(200.0);
            
            ui.heading(egui::RichText::new("PAUSED").size(48.0));
            ui.add_space(40.0);
            
            if ui.add_sized([200.0, 50.0], egui::Button::new(egui::RichText::new("Resume").size(20.0))).clicked() 
                || keyboard_input.just_pressed(KeyCode::Escape) {
                game_state.set(GameState::Playing);
            }
            
            ui.add_space(20.0);
            
            if ui.add_sized([200.0, 50.0], egui::Button::new(egui::RichText::new("Main Menu").size(20.0))).clicked() {
                game_state.set(GameState::MainMenu);
            }
        });
    });
}

pub fn game_over_screen(
    mut contexts: EguiContexts,
    mut game_state: ResMut<NextState<GameState>>,
    game_stats: Res<GameStats>,
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
            ui.label(egui::RichText::new(format!("💰 Coins Earned: {}", coins_earned)).size(20.0).color(egui::Color32::YELLOW));
            
            ui.add_space(40.0);
            
            if ui.add_sized([200.0, 50.0], egui::Button::new(egui::RichText::new("Play Again").size(20.0))).clicked() {
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
                
                if ui.add_enabled(can_afford, egui::Button::new(format!("Upgrade (💰 {})", cost))).clicked() {
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
                
                if ui.add_enabled(can_afford, egui::Button::new(format!("Upgrade (💰 {})", cost))).clicked() {
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
                
                if ui.add_enabled(can_afford, egui::Button::new(format!("Upgrade (💰 {})", cost))).clicked() {
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
                
                if ui.add_enabled(can_afford, egui::Button::new(format!("Upgrade (💰 {})", cost))).clicked() {
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