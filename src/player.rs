use std::collections::HashMap;

use shipyard::{EntityId, View, IntoIter, IntoWithId};

use crate::utils::WorldGet;
use crate::{State, RunState};
use crate::map::{Map, TileType};
use crate::components::{Position, CombatStats, Item, WantsToPickupItem, Fire, SpatialKnowledge, Viewshed};

pub fn get_player_map_knowledge(gs: &State) -> HashMap<usize, (TileType, Vec<EntityId>)>{
    let world = &gs.world;
    let player_id = gs.get_player().0;//res.get::<EntityId>().unwrap();

    if let Ok(space) =  world.get::<SpatialKnowledge>(player_id) {
        space.tiles.clone()
    } else {
        HashMap::new()
    }
}

pub fn get_player_viewshed(gs: &State) -> Viewshed {
    let world = &gs.world;
    let player_id = gs.get_player().0;//res.get::<EntityId>().unwrap();

    let vs = world.get::<Viewshed>(player_id).unwrap();

    vs.clone()
}

pub fn get_item(gs: &mut State){
    let player_id = gs.get_player().0;//res.get::<EntityId>().unwrap();
    let player_pos = gs.get_player_pos().0;//res.get::<Point>().unwrap();
    let mut log = gs.get_log();//res.get_mut::<GameLog>().unwrap();

    let mut target_item: Option<EntityId> = None;

    gs.world.run(|vpos: View<Position>, vitem: View<Item>| {
        for (id, (pos, item)) in (&vpos, &vitem).iter().with_id() {
            let pos = pos.ps.first().unwrap();
            if pos.x == player_pos.x && pos.y == player_pos.y {
                target_item = Some(id);
            }
        }
    });

    match target_item {
        None => {log.messages.push(format!("There is nothing to pick up here"))}
        Some(item) => {
            let _res = gs.world.add_component(player_id, WantsToPickupItem {
                collected_by: player_id,
                item
            });
        }
    }
}

pub fn reveal_map(gs: &mut State){
    let world = &gs.world;
    // let res = &gs.resources;
    let map: &mut Map = &mut gs.get_map();//&mut res.get_mut::<Map>().unwrap();
    let player_id = gs.get_player().0;//res.get::<EntityId>().unwrap();


    if let Ok(mut space) =  world.get::<SpatialKnowledge>(player_id) {
        for i in 0..map.tiles.len() {
            space.tiles.insert(i, (map.tiles[i], map.tile_content[i].clone()));
        }
    }
}

pub fn try_next_level(gs: &mut State) -> bool {
    let player_pos = gs.get_player_pos().0;//res.get::<Point>().unwrap();
    let map = gs.get_map();//res.get::<Map>().unwrap();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::StairsDown {
        true
    }
    else {
        let mut log = gs.get_log();//res.get_mut::<GameLog>().unwrap();
        log.messages.push(format!("There is no stairs down here"));
        false
    }
}

pub fn skip_turn(gs: &mut State) -> RunState {
    let player_id = gs.get_player().0;//res.get::<EntityId>().unwrap();
    let mut stats = gs.world.get::<CombatStats>(player_id).unwrap();

    // regen player if not on fire
    match gs.world.get::<Fire>(player_id) {
        Ok(_) => {},
        Err(_) => stats.hp = i32::min(stats.hp + stats.regen_rate, stats.max_hp),
    }

    RunState::PlayerTurn
}