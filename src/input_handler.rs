use crate::{State, RunState, GameMode, movement, entity_factory, player, utils::{dir_to_point}};
use hecs::Entity;
use rltk::{Rltk, VirtualKeyCode};


pub fn handle_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    let game_mode: GameMode = *gs.resources.get::<GameMode>().unwrap();

    let player_id: Entity = *gs.resources.get::<Entity>().unwrap();

    // hold shift to move by 10 squares at a time
    let mut movemod = 1;
    if ctx.shift && game_mode == GameMode::Sim {
        movemod = 10;
    }

    match game_mode {
        GameMode::NotSelected => unreachable!(), 
        GameMode::Sim => {
            match ctx.key {
                None => { return RunState::AwaitingInput }
                Some(key) => match key {
                    VirtualKeyCode::Left => movement::try_move_entity(player_id, dir_to_point(4, movemod), gs),
                    VirtualKeyCode::Right => movement::try_move_entity(player_id, dir_to_point( 6, movemod), gs),
                    VirtualKeyCode::Up => movement::try_move_entity(player_id, dir_to_point( 8, movemod), gs),
                    VirtualKeyCode::Down => movement::try_move_entity(player_id, dir_to_point( 2, movemod), gs),
                    VirtualKeyCode::Y => movement::try_move_entity(player_id, dir_to_point( 7, movemod), gs),
                    VirtualKeyCode::U => movement::try_move_entity(player_id, dir_to_point( 9, movemod), gs),
                    VirtualKeyCode::N => movement::try_move_entity(player_id, dir_to_point( 3, movemod), gs),
                    VirtualKeyCode::B => movement::try_move_entity(player_id, dir_to_point( 1, movemod), gs),
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
                    VirtualKeyCode::Left => movement::try_move_entity(player_id, dir_to_point( 4, movemod), gs),
                    VirtualKeyCode::Right => movement::try_move_entity(player_id, dir_to_point( 6, movemod), gs),
                    VirtualKeyCode::Up => movement::try_move_entity(player_id, dir_to_point( 8, movemod), gs),
                    VirtualKeyCode::Down => movement::try_move_entity(player_id, dir_to_point( 2, movemod), gs),
                    VirtualKeyCode::Y => movement::try_move_entity(player_id, dir_to_point( 7, movemod), gs),
                    VirtualKeyCode::U => movement::try_move_entity(player_id, dir_to_point( 9, movemod), gs),
                    VirtualKeyCode::N => movement::try_move_entity(player_id, dir_to_point( 3, movemod), gs),
                    VirtualKeyCode::B => movement::try_move_entity(player_id, dir_to_point( 1, movemod), gs),
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