use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
    GameOver,
    UpgradeShop,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum GameMode {
    #[default]
    FreePlay,
    TimeAttack,
    TargetHunt,
    Survival,
    RaceTheClock,
}

#[derive(Resource)]
pub struct CurrentGameMode {
    pub mode: GameMode,
}

impl Default for CurrentGameMode {
    fn default() -> Self {
        Self {
            mode: GameMode::FreePlay,
        }
    }
}

#[derive(Resource)]
pub struct GameStats {
    pub score: u32,
    pub high_score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub targets_hit: u32,
    pub time_played: f32,
    pub coins: u32,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            score: 0,
            high_score: 0,
            combo: 0,
            max_combo: 0,
            targets_hit: 0,
            time_played: 0.0,
            coins: 0,
        }
    }
}

#[derive(Resource)]
pub struct ChallengeTimer {
    pub time_remaining: f32,
    pub total_time: f32,
}

impl Default for ChallengeTimer {
    fn default() -> Self {
        Self {
            time_remaining: 60.0,
            total_time: 60.0,
        }
    }
}

#[derive(Resource)]
pub struct UpgradeData {
    pub speed_level: u32,
    pub maneuverability_level: u32,
    pub magnet_level: u32,
    pub multiplier_level: u32,
}

impl Default for UpgradeData {
    fn default() -> Self {
        Self {
            speed_level: 1,
            maneuverability_level: 1,
            magnet_level: 0,
            multiplier_level: 1,
        }
    }
}

pub fn get_upgrade_cost(level: u32) -> u32 {
    100 * level * level
}

pub fn get_speed_bonus(level: u32) -> f32 {
    1.0 + (level as f32 - 1.0) * 0.2
}

pub fn get_maneuverability_bonus(level: u32) -> f32 {
    1.0 + (level as f32 - 1.0) * 0.15
}

pub fn get_magnet_range(level: u32) -> f32 {
    level as f32 * 5.0
}

pub fn get_score_multiplier(level: u32) -> u32 {
    level
}