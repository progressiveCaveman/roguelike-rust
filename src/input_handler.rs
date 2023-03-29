use std::collections::HashMap;

use crate::{State, RunState, GameMode, entity_factory, player, utils::{dir_to_point}, effects::{add_effect, EffectType, Targets}};
use rltk::{Rltk, VirtualKeyCode};

pub fn handle_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    let game_mode = gs.get_game_mode();//*gs.resources.get::<GameMode>().unwrap();
    let map = gs.get_map();//*gs.resources.get::<GameMode>().unwrap();

    let player_id = gs.get_player().0;//*gs.resources.get::<EntityId>().unwrap();
    let player_pos = gs.get_player_pos().0;//*gs.resources.get::<EntityId>().unwrap();

    // hold shift to move by 10 squares at a time
    let mut movemod = 1;
    if ctx.shift && *game_mode == GameMode::Sim {
        movemod = 10;
    }

    let dir_targets: HashMap<i32, usize> = HashMap::new();
    dir_targets[&1] = map.point_idx(dir_to_point(player_pos, 1, movemod));
    dir_targets[&2] = map.point_idx(dir_to_point(player_pos, 2, movemod));
    dir_targets[&3] = map.point_idx(dir_to_point(player_pos, 3, movemod));
    dir_targets[&4] = map.point_idx(dir_to_point(player_pos, 4, movemod));
    dir_targets[&6] = map.point_idx(dir_to_point(player_pos, 6, movemod));
    dir_targets[&7] = map.point_idx(dir_to_point(player_pos, 7, movemod));
    dir_targets[&8] = map.point_idx(dir_to_point(player_pos, 8, movemod));
    dir_targets[&9] = map.point_idx(dir_to_point(player_pos, 9, movemod));

    match game_mode {
        GameMode::NotSelected => unreachable!(), 
        GameMode::Sim => {
            match ctx.key {
                None => { return RunState::AwaitingInput }
                Some(key) => match key {
                    VirtualKeyCode::Left => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&4]}),
                    VirtualKeyCode::Right => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&6]}),
                    VirtualKeyCode::Up => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&8]}),
                    VirtualKeyCode::Down => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&2]}),
                    VirtualKeyCode::Y => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&7]}),
                    VirtualKeyCode::U => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&9]}),
                    VirtualKeyCode::N => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&3]}),
                    VirtualKeyCode::B => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&1]}),
                    VirtualKeyCode::W => return RunState::PlayerTurn,
                    VirtualKeyCode::Escape => return RunState::SaveGame,
                    VirtualKeyCode::Space => gs.autorun = !gs.autorun,
                    _ => { return RunState::AwaitingInput }
                }
            }
            RunState::AwaitingInput   
        },
        GameMode::RL => {
            // let player_id: EntityId = *gs.resources.get::<EntityId>().unwrap();

            match ctx.key {
                None => { return RunState::AwaitingInput }
                Some(key) => match key {
                    VirtualKeyCode::Left => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&4]}),
                    VirtualKeyCode::Right => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&6]}),
                    VirtualKeyCode::Up => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&8]}),
                    VirtualKeyCode::Down => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&2]}),
                    VirtualKeyCode::Y => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&7]}),
                    VirtualKeyCode::U => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&9]}),
                    VirtualKeyCode::N => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&3]}),
                    VirtualKeyCode::B => add_effect(Some(player_id), EffectType::Move {}, Targets::Tile { tile_idx: dir_targets[&1]}),
                    VirtualKeyCode::G => player::get_item(gs),
                    VirtualKeyCode::X => add_effect(Some(player_id), EffectType::Explore {}, Targets::Single { target: player_id }),
                    VirtualKeyCode::R => player::reveal_map(gs),
                    VirtualKeyCode::F => return RunState::ShowTargeting { range: 6, item: entity_factory::tmp_fireball(&mut gs.world) },
                    VirtualKeyCode::I => return RunState::ShowInventory,
                    VirtualKeyCode::W => return player::skip_turn(gs),
                    VirtualKeyCode::Escape => return RunState::SaveGame,
                    VirtualKeyCode::Period => {
                        if player::try_next_level(gs) { return RunState::NextLevel; }
                    }
                    _ => { return RunState::AwaitingInput }
                }
            }
            RunState::PlayerTurn    
        },
    }
}