use std::collections::HashMap;

use rltk::{Point, DijkstraMap};
use hecs::*;
use resources::*;

use crate::{State, RunState};
use crate::map::{Map, TileType};
use crate::components::{Position, CombatStats, Item, WantsToPickupItem, Fire, SpatialKnowledge, Viewshed};
use crate::gamelog::GameLog;
use crate::movement::try_move_entity;

use crate::dijkstra_utils::dijkstra_backtrace;

pub fn get_player_map_knowledge(gs: &State) -> HashMap<usize, (TileType, Vec<Entity>)>{
    let world = &gs.world;
    let res = &gs.resources;
    let player_id = res.get::<Entity>().unwrap();

    if let Ok(space) =  world.get_mut::<SpatialKnowledge>(*player_id) {
        space.tiles.clone()
    } else {
        HashMap::new()
    }
}

pub fn get_player_viewshed(gs: &State) -> Viewshed {
    let world = &gs.world;
    let res = &gs.resources;
    let player_id = res.get::<Entity>().unwrap();

    let vs = world.get_mut::<Viewshed>(*player_id).unwrap();

    vs.clone()
    // if let Ok(vs) = world.get_mut::<Viewshed>(*player_id).unwrap() {
    //     &vs
    // } else {
    //     unreachable!()
    // }
}

pub fn get_item(world: &mut World, res: &mut Resources){
    let player_id = res.get::<Entity>().unwrap();
    let player_pos = res.get::<Point>().unwrap();
    let mut log = res.get_mut::<GameLog>().unwrap();

    let mut target_item: Option<Entity> = None;

    for (id, (_item, pos)) in &mut world.query::<(&Item, &Position)>() {
        let pos = pos.ps.first().unwrap();
        if pos.x == player_pos.x && pos.y == player_pos.y {
            target_item = Some(id);
        }
    }

    match target_item {
        None => {log.messages.push(format!("There is nothing to pick up here"))}
        Some(item) => {
            let _res = world.insert_one(*player_id, WantsToPickupItem {
                collected_by: *player_id,
                item
            });
        }
    }
}

pub fn autoexplore(gs: &mut State){
    let player_pos: Point;

    // TODO Check for adjacent enemies and attack them
    
    // Use djikstras to find nearest unexplored tile
    let mut target = (0, std::f32::MAX); // tile_idx, distance
    let dijkstra_map: DijkstraMap;
    {
        let res = &gs.resources;
        player_pos = res.get::<Point>().unwrap().clone();
        let map: &mut Map = &mut res.get_mut::<Map>().unwrap();
        let mut log = res.get_mut::<GameLog>().unwrap();
        let player_idx = map.xy_idx(player_pos.x, player_pos.y);
        let player_knowledge = get_player_map_knowledge(gs);
        let starts = vec![player_idx];
        dijkstra_map = rltk::DijkstraMap::new(map.width, map.height, &starts, map, 200.0);
        for (i, tile) in map.tiles.iter().enumerate() {
            if *tile != TileType::Wall && !player_knowledge.contains_key(&i) {
                let distance_to_start = dijkstra_map.map[i];

                if distance_to_start < target.1 {
                    target = (i, distance_to_start)
                }
            }
        }

        if target.1 == std::f32::MAX {
            log.messages.push(format!("No tiles left to explore"));
            return;
        }

        log.messages.push(format!("Closest unexplored tile is {} steps away", target.1));

        map.dijkstra_map = dijkstra_map.map.clone();

        // We have a target tile. Now follow the path up the chain
        let t = dijkstra_backtrace(dijkstra_map, map, player_idx, target.0);
        target = (t, 1.0);
    }
    
    // Send a move command
    let dx: i32;
    let dy: i32;
    {
        let res = &gs.resources;
        let map = res.get::<Map>().unwrap();
        let targetx = map.idx_xy(target.0).0;
        let targety = map.idx_xy(target.0).1;
        dx = targetx - player_pos.x;
        // if dx != 0 { dx = dx/dx.abs(); }
        dy = targety - player_pos.y;
        // if dy != 0 { dy = dy / dy.abs(); }
    }
    
    let player_id: Entity = *gs.resources.get::<Entity>().unwrap();
    try_move_entity(player_id, dx, dy, gs);
}

pub fn reveal_map(gs: &mut State){
    let world = &gs.world;
    let res = &gs.resources;
    let map: &mut Map = &mut res.get_mut::<Map>().unwrap();
    let player_id = res.get::<Entity>().unwrap();


    if let Ok(mut space) =  world.get_mut::<SpatialKnowledge>(*player_id) {
        for i in 0..map.tiles.len() {
            space.tiles.insert(i, (map.tiles[i], map.tile_content[i].clone()));
        }
    }
}

pub fn try_next_level(_world: &mut World, res: &mut Resources) -> bool {
    let player_pos = res.get::<Point>().unwrap();
    let map = res.get::<Map>().unwrap();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::StairsDown {
        true
    }
    else {
        let mut log = res.get_mut::<GameLog>().unwrap();
        log.messages.push(format!("There is no stairs down here"));
        false
    }
}

pub fn skip_turn(world: &mut World, res: &mut Resources) -> RunState {
    let player_id = res.get::<Entity>().unwrap();
    let mut stats = world.get_mut::<CombatStats>(*player_id).unwrap();

    // regen player if not on fire
    // let fire = world.get_mut::<Fire>(*player_id);
    match world.get_mut::<Fire>(*player_id) {
        Ok(_) => {},
        Err(_) => stats.hp = i32::min(stats.hp + stats.regen_rate, stats.max_hp),
    }

    RunState::PlayerTurn
}

// pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
//     let player_id: Entity = *gs.resources.get::<Entity>().unwrap();

//     match ctx.key {
//         None => { return RunState::AwaitingInput }
//         Some(key) => match key {
//             VirtualKeyCode::Left => try_move_entity(player_id, -1, 0, gs),
//             VirtualKeyCode::Right => try_move_entity(player_id, 1, 0, gs),
//             VirtualKeyCode::Up => try_move_entity(player_id, 0, -1, gs),
//             VirtualKeyCode::Down => try_move_entity(player_id, 0, 1, gs),
//             VirtualKeyCode::Y => try_move_entity(player_id, -1, -1, gs),
//             VirtualKeyCode::U => try_move_entity(player_id, 1, -1, gs),
//             VirtualKeyCode::N => try_move_entity(player_id, 1, 1, gs),
//             VirtualKeyCode::B => try_move_entity(player_id, -1, 1, gs),
//             VirtualKeyCode::G => get_item(&mut gs.world, &mut gs.resources),
//             VirtualKeyCode::X => autoexplore(gs),
//             VirtualKeyCode::R => reveal_map(gs),
//             VirtualKeyCode::F => return RunState::ShowTargeting { range: 6, item: entity_factory::tmp_fireball(&mut gs.world) },
//             VirtualKeyCode::I => return RunState::ShowInventory,
//             VirtualKeyCode::W => return skip_turn(&mut gs.world, &mut gs.resources),
//             VirtualKeyCode::Escape => return RunState::SaveGame,
//             VirtualKeyCode::Period => {
//                 if try_next_level(&mut gs.world, &mut gs.resources) { return RunState::NextLevel; }
//             }
//             _ => { return RunState::AwaitingInput }
//         }
//     }
//     RunState::PlayerTurn
// }
