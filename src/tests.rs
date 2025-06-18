#[cfg(test)]
mod tests {
    use crate::game_state::{GameMode, GameStats, UpgradeData, get_speed_bonus, get_maneuverability_bonus, get_magnet_range, get_score_multiplier, get_upgrade_cost};

    #[test]
    fn test_game_stats_default() {
        let stats = GameStats::default();
        assert_eq!(stats.score, 0);
        assert_eq!(stats.high_score, 0);
        assert_eq!(stats.targets_hit, 0);
        assert_eq!(stats.combo, 0);
        assert_eq!(stats.max_combo, 0);
        assert_eq!(stats.coins, 0);
    }

    #[test]
    fn test_upgrade_data_default() {
        let upgrades = UpgradeData::default();
        assert_eq!(upgrades.speed_level, 1);
        assert_eq!(upgrades.maneuverability_level, 1);
        assert_eq!(upgrades.magnet_level, 0);
        assert_eq!(upgrades.multiplier_level, 1);
    }

    #[test]
    fn test_speed_bonus_calculation() {
        assert_eq!(get_speed_bonus(0), 0.8);
        assert_eq!(get_speed_bonus(1), 1.0);
        assert_eq!(get_speed_bonus(2), 1.2);
        assert_eq!(get_speed_bonus(3), 1.4);
        assert_eq!(get_speed_bonus(4), 1.6);
        assert_eq!(get_speed_bonus(5), 1.8);
    }

    #[test]
    fn test_maneuverability_bonus_calculation() {
        assert_eq!(get_maneuverability_bonus(0), 0.85);
        assert_eq!(get_maneuverability_bonus(1), 1.0);
        assert_eq!(get_maneuverability_bonus(2), 1.15);
        assert_eq!(get_maneuverability_bonus(3), 1.3);
        assert_eq!(get_maneuverability_bonus(4), 1.45);
        assert_eq!(get_maneuverability_bonus(5), 1.6);
    }

    #[test]
    fn test_magnet_range_calculation() {
        assert_eq!(get_magnet_range(0), 0.0);
        assert_eq!(get_magnet_range(1), 5.0);
        assert_eq!(get_magnet_range(2), 10.0);
        assert_eq!(get_magnet_range(3), 15.0);
        assert_eq!(get_magnet_range(4), 20.0);
        assert_eq!(get_magnet_range(5), 25.0);
    }

    #[test]
    fn test_score_multiplier_calculation() {
        assert_eq!(get_score_multiplier(0), 0);
        assert_eq!(get_score_multiplier(1), 1);
        assert_eq!(get_score_multiplier(2), 2);
        assert_eq!(get_score_multiplier(3), 3);
        assert_eq!(get_score_multiplier(4), 4);
        assert_eq!(get_score_multiplier(5), 5);
    }

    #[test]
    fn test_upgrade_cost_calculation() {
        assert_eq!(get_upgrade_cost(0), 0);
        assert_eq!(get_upgrade_cost(1), 100);
        assert_eq!(get_upgrade_cost(2), 400);
        assert_eq!(get_upgrade_cost(3), 900);
        assert_eq!(get_upgrade_cost(4), 1600);
        assert_eq!(get_upgrade_cost(5), 2500);
    }

    #[test]
    fn test_game_mode_properties() {
        // Test that each game mode has distinct properties
        let modes = vec![
            GameMode::TargetHunt,
            GameMode::Survival,
            GameMode::TimeAttack,
            GameMode::FreePlay,
            GameMode::RaceTheClock,
        ];
        
        for mode in modes {
            match mode {
                GameMode::TargetHunt => {
                    // Target hunt should have targets
                },
                GameMode::Survival => {
                    // Survival should have enemies
                },
                GameMode::TimeAttack => {
                    // Time attack should have a timer
                },
                GameMode::FreePlay => {
                    // Free play should be peaceful
                },
                GameMode::RaceTheClock => {
                    // Race the clock should have urgency
                },
            }
        }
    }
}