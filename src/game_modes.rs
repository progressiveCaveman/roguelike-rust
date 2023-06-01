use engine::{GameSettings, GameMode};

pub fn get_settings(mode: GameMode) -> GameSettings {
    match mode {
        GameMode::NotSelected => unreachable!(),
        GameMode::Sim => {
            GameSettings { 
                mode, 
                mapsize: (200, 80),
                follow_player: false,
                use_player_los: false, 
            }
        },
        GameMode::RL => {
            GameSettings { 
                mode, 
                mapsize: (80, 40),
                follow_player: true,
                use_player_los: true, 
            }
        },
    }
}