use std::collections::HashMap;

use shipyard::{EntityId, UniqueView, UniqueViewMut, ViewMut, Get};

use crate::gamelog::GameLog;
use crate::utils::{PlayerID, PPoint};
use crate::{State};
use crate::map::{Map, TileType};
use crate::components::{SpatialKnowledge, Viewshed};

pub fn get_player_map_knowledge(gs: &State) -> HashMap<usize, (TileType, Vec<EntityId>)>{
    let world = &gs.world;
    let player_id = gs.world.borrow::<UniqueView<PlayerID>>().unwrap().0;

    return if let Ok(space) = world.borrow::<ViewMut<SpatialKnowledge>>() { //world.get::<SpatialKnowledge>(player_id) {
        space.get(player_id).unwrap().tiles.clone()
    } else {
        HashMap::new()
    }
}

pub fn get_player_viewshed(gs: &State) -> Viewshed {
    let world = &gs.world;
    let player_id = gs.world.borrow::<UniqueView<PlayerID>>().unwrap().0;

    let vs = world.borrow::<ViewMut<Viewshed>>().unwrap();

    vs.get(player_id).unwrap().clone()
}

pub fn reveal_map(gs: &State){
    // let world = &gs.world;
    // let res = &gs.resources;
    let map = gs.world.borrow::<UniqueView<Map>>().unwrap();
    let player_id = gs.world.borrow::<UniqueView<PlayerID>>().unwrap().0;

    if let Ok(mut vspace) = gs.world.borrow::<ViewMut<SpatialKnowledge>>() {
        if let Ok(space) = (&mut vspace).get(player_id) {
            for i in 0..map.tiles.len() {
                space.tiles.insert(i, (map.tiles[i], map.tile_content[i].clone()));
            }
        }
    }
}

pub fn try_next_level(gs: &State) -> bool {
    let player_pos = gs.world.borrow::<UniqueView<PPoint>>().unwrap().0;
    let map = gs.world.borrow::<UniqueView<Map>>().unwrap();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::StairsDown {
        true
    }
    else {
        let mut log = gs.world.borrow::<UniqueViewMut<GameLog>>().unwrap();
        log.messages.push(format!("There is no stairs down here"));
        false
    }
}