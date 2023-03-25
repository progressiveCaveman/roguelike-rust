use std::collections::HashMap;

use rltk::{Point};
use resources::*;
use shipyard::{EntityId, World, View, IntoIter, IntoWithId};

use crate::utils::WorldGet;
use crate::{State, RunState};
use crate::map::{Map, TileType};
use crate::components::{Position, CombatStats, Item, WantsToPickupItem, Fire, SpatialKnowledge, Viewshed};
use crate::gamelog::GameLog;

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

pub fn get_item(world: &mut World, res: &mut Resources){
    let player_id = res.get::<EntityId>().unwrap();
    let player_pos = res.get::<Point>().unwrap();
    let mut log = res.get_mut::<GameLog>().unwrap();

    let mut target_item: Option<EntityId> = None;

    world.run(|vpos: View<Position>, vitem: View<Item>| {
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
            let _res = world.add_component(*player_id, WantsToPickupItem {
                collected_by: *player_id,
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
    let player_id = res.get::<EntityId>().unwrap();
    let mut stats = world.get::<CombatStats>(*player_id).unwrap();

    // regen player if not on fire
    match world.get::<Fire>(*player_id) {
        Ok(_) => {},
        Err(_) => stats.hp = i32::min(stats.hp + stats.regen_rate, stats.max_hp),
    }

    RunState::PlayerTurn
}