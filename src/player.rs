use std::collections::HashMap;

use rltk::{Point};
use hecs::*;
use resources::*;

use crate::{State, RunState};
use crate::map::{Map, TileType};
use crate::components::{Position, CombatStats, Item, WantsToPickupItem, Fire, SpatialKnowledge, Viewshed};
use crate::gamelog::GameLog;

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