use crate::{State, RunState, GameMode, movement, entity_factory, player};
use hecs::Entity;
use rltk::{Rltk, VirtualKeyCode};


pub fn handle_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    let game_mode: GameMode = *gs.resources.get::<GameMode>().unwrap();

    match game_mode {
        GameMode::NotSelected => unreachable!(), 
        GameMode::Sim => {
            let player_id: Entity = *gs.resources.get::<Entity>().unwrap();

            let mut movement_mod = 1;
            if ctx.shift {
                movement_mod = 10;
            }

            match ctx.key {
                None => { return RunState::AwaitingInput }
                Some(key) => match key {
                    VirtualKeyCode::Left => movement::try_move_entity(player_id, -1 * movement_mod, 0, gs),
                    VirtualKeyCode::Right => movement::try_move_entity(player_id, 1 * movement_mod, 0, gs),
                    VirtualKeyCode::Up => movement::try_move_entity(player_id, 0, -1 * movement_mod, gs),
                    VirtualKeyCode::Down => movement::try_move_entity(player_id, 0, 1 * movement_mod, gs),
                    VirtualKeyCode::Y => movement::try_move_entity(player_id, -1 * movement_mod, -1 * movement_mod, gs),
                    VirtualKeyCode::U => movement::try_move_entity(player_id, 1 * movement_mod, -1 * movement_mod, gs),
                    VirtualKeyCode::N => movement::try_move_entity(player_id, 1 * movement_mod, 1 * movement_mod, gs),
                    VirtualKeyCode::B => movement::try_move_entity(player_id, -1 * movement_mod, 1 * movement_mod, gs),
                    VirtualKeyCode::W => return RunState::PlayerTurn,
                    VirtualKeyCode::Escape => return RunState::SaveGame,
                    VirtualKeyCode::Space => gs.autorun = !gs.autorun,
                    _ => { return RunState::AwaitingInput }
                }
            }
            RunState::AwaitingInput   
        },
        GameMode::RL => {
            let player_id: Entity = *gs.resources.get::<Entity>().unwrap();

            match ctx.key {
                None => { return RunState::AwaitingInput }
                Some(key) => match key {
                    VirtualKeyCode::Left => movement::try_move_entity(player_id, -1, 0, gs),
                    VirtualKeyCode::Right => movement::try_move_entity(player_id, 1, 0, gs),
                    VirtualKeyCode::Up => movement::try_move_entity(player_id, 0, -1, gs),
                    VirtualKeyCode::Down => movement::try_move_entity(player_id, 0, 1, gs),
                    VirtualKeyCode::Y => movement::try_move_entity(player_id, -1, -1, gs),
                    VirtualKeyCode::U => movement::try_move_entity(player_id, 1, -1, gs),
                    VirtualKeyCode::N => movement::try_move_entity(player_id, 1, 1, gs),
                    VirtualKeyCode::B => movement::try_move_entity(player_id, -1, 1, gs),
                    VirtualKeyCode::G => player::get_item(&mut gs.world, &mut gs.resources),
                    VirtualKeyCode::X => movement::autoexplore(gs, player_id),
                    VirtualKeyCode::R => player::reveal_map(gs),
                    VirtualKeyCode::F => return RunState::ShowTargeting { range: 6, item: entity_factory::tmp_fireball(&mut gs.world) },
                    VirtualKeyCode::I => return RunState::ShowInventory,
                    VirtualKeyCode::W => return player::skip_turn(&mut gs.world, &mut gs.resources),
                    VirtualKeyCode::Escape => return RunState::SaveGame,
                    VirtualKeyCode::Period => {
                        if player::try_next_level(&mut gs.world, &mut gs.resources) { return RunState::NextLevel; }
                    }
                    _ => { return RunState::AwaitingInput }
                }
            }
            RunState::PlayerTurn    
        },
    }
}

// pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
// }
