use std::collections::HashMap;

use crate::{State, RunState, GameMode, entity_factory, player, utils::{dir_to_point, PPoint, PlayerID, AutoRun}, effects::{add_effect, EffectType, Targets}, map::Map};
use rltk::{Rltk, VirtualKeyCode};
use shipyard::{AllStoragesViewMut, UniqueView, UniqueViewMut};

pub fn handle_input(gs: &State, ctx: &Rltk) -> RunState {
    let game_mode = gs.world.borrow::<UniqueView<GameMode>>().unwrap();
    let map = gs.world.borrow::<UniqueView<Map>>().unwrap();
    let mut autorun = gs.world.borrow::<UniqueViewMut<AutoRun>>().unwrap();

    let player_id = gs.world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
    let player_pos = gs.world.borrow::<UniqueView<PPoint>>().unwrap().0;
    let player_pos_idx = map.point_idx(player_pos);

    // hold shift to move by 10 squares at a time
    let mut movemod = 1;
    if ctx.shift && *game_mode == GameMode::Sim {
        movemod = 10;
    }

    let mut dir_targets: HashMap<i32, usize> = HashMap::new();
    dir_targets.insert(1, map.point_idx(dir_to_point(player_pos, 1, movemod)));
    dir_targets.insert(1, map.point_idx(dir_to_point(player_pos, 1, movemod)));
    dir_targets.insert(2, map.point_idx(dir_to_point(player_pos, 2, movemod)));
    dir_targets.insert(3, map.point_idx(dir_to_point(player_pos, 3, movemod)));
    dir_targets.insert(4, map.point_idx(dir_to_point(player_pos, 4, movemod)));
    dir_targets.insert(6, map.point_idx(dir_to_point(player_pos, 6, movemod)));
    dir_targets.insert(7, map.point_idx(dir_to_point(player_pos, 7, movemod)));
    dir_targets.insert(8, map.point_idx(dir_to_point(player_pos, 8, movemod)));
    dir_targets.insert(9, map.point_idx(dir_to_point(player_pos, 9, movemod)));

    match *game_mode {
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
                    VirtualKeyCode::Space => autorun.0 = !autorun.0,
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
                    VirtualKeyCode::G => add_effect(Some(player_id), EffectType::PickUp {}, Targets::Tile { tile_idx: player_pos_idx}),//player::get_item(gs),
                    VirtualKeyCode::X => add_effect(Some(player_id), EffectType::Explore {}, Targets::Single { target: player_id }),
                    VirtualKeyCode::R => player::reveal_map(gs),
                    VirtualKeyCode::F => return RunState::ShowTargeting { range: 6, item: gs.world.run(|mut store: AllStoragesViewMut|{entity_factory::tmp_fireball(&mut store)}) },
                    VirtualKeyCode::I => return RunState::ShowInventory,
                    VirtualKeyCode::W => add_effect(Some(player_id), EffectType::Wait {}, Targets::Single { target: player_id }),//return player::skip_turn(gs),
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