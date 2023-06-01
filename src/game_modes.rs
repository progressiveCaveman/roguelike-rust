use engine::{GameSettings, GameMode};

pub fn get_settings(mode: GameMode) -> GameSettings {
    match mode {
        GameMode::NotSelected => unreachable!(),
        GameMode::Sim => {
            GameSettings { 
                mode, 
                mapsize: (200, 80) 
            }
        },
        GameMode::RL => {
            GameSettings { 
                mode, 
                mapsize: (80, 40) 
            }
        },
    }
}