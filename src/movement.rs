use hecs::*;
use rltk::{Point, DijkstraMap, NavigationPath};

use crate::utils::{dijkstra_backtrace, InvalidPoint, point_plus};
use crate::{State, GameMode};
use crate::map::{Map, TileType};
use crate::components::{Position, Player, Viewshed, CombatStats, WantsToAttack, BlocksTile, SpatialKnowledge, Locomotive, LocomotionType};

/// dp is delta
pub fn try_move_entity(entity: Entity, dp: Point, gs: &mut State) {
    let mut map = gs.resources.get_mut::<Map>().unwrap();
    let mode = gs.resources.get::<GameMode>().unwrap();
    let mut needs_wants_to_attack: Option<(Entity, WantsToAttack)> = None;

    // if tp.x < 0 || tp.y < 0 || tp.x >= map.width || tp.y >= map.height { return; }

    if let Ok(mut pos) = gs.world.get_mut::<Position>(entity) {
        // if let Ok(_loc) = &gs.world.get::<Locomotive>(entity) {

            let canmove = can_move(&gs.world, &map, entity, &pos, dp);

            // In Sim mode, player is basically just a camera object
            let mut is_camera = false;
            if let Ok(_) = &gs.world.get::<Player>(entity) {
                if *mode == GameMode::Sim { is_camera = true; }
            }

            if !is_camera {
                if let Some(target) = get_target(&gs.world, &map, entity, &pos, dp) {
                    needs_wants_to_attack = Some((entity, WantsToAttack {target}));

                    // gs.world.insert_one(entity, WantsToAttack {target});
                    // return;
                }
            }

            // do movement
            if is_camera || canmove {                
                if let Ok(mut vs) = gs.world.get_mut::<Viewshed>(entity) {
                    vs.dirty = true;
                }

                // for pos in pos.ps.iter_mut() {
                for i in 0..pos.ps.len() {
                    let oldidx = map.point_idx(pos.ps[i]);

                    pos.ps[i] = point_plus(pos.ps[i], dp);

                    let idx = map.point_idx(pos.ps[i]);
                    map.blocked[oldidx] = false;
                    map.blocked[idx] = true;
                }
    
                // If this is a player, change the position in resources according to first in pos.ps
                match &gs.world.get_mut::<Player>(entity) {
                    Err(_e) => {},
                    Ok(_player) => {
                        let mut ppos = gs.resources.get_mut::<rltk::Point>().unwrap();
                        ppos.x = pos.ps[0].x;
                        ppos.y = pos.ps[0].y;
                    }
                }

                return;
            }
        // }
    }

    if let Some((e, c)) = needs_wants_to_attack {
        let _res = gs.world.insert_one(e, c);
    }
}

// checks for entities with combat stats on block
pub fn get_target(world: &World, map: &Map, entity: Entity, pos: &Position, dp: Point) -> Option<Entity> {

    // check for combat stats on entity
    if let Err(_) = world.get::<CombatStats>(entity){
        return None;
    }

    for pos in pos.ps.iter() {
        let dest_idx = map.xy_idx(pos.x + dp.x, pos.y + dp.y);
        // let dest_idx = map.point_idx(tp);
        for potential_target in map.tile_content[dest_idx].iter() {
            if *potential_target == entity {
                continue;
            }

            match &world.get::<CombatStats>(*potential_target) {
                Ok(_cs) => {
                    return Some(*potential_target)
                }
                Err(_e) => {}
            }
        }
    }

    None
}

pub fn can_move(world: &World, map: &Map, entity: Entity, pos: &Position, dp: Point) -> bool {
    if let Ok(loco) = world.get::<Locomotive>(entity){
        for pos in pos.ps.iter() {
            // check for tiles that block
            let dest_idx = map.xy_idx(pos.x + dp.x, pos.y + dp.y);
            // let dest_idx = map.point_idx(tp);
            if loco.mtype == LocomotionType::Ground && map.blocks_movement(dest_idx) { 
                return false;
            }

            if loco.mtype == LocomotionType::Water && map.tiles[dest_idx] != TileType::Water { 
                return false;
            }

            // check for entities that block
            for potential_target in map.tile_content[dest_idx].iter() {
                if *potential_target == entity {
                    continue;
                }
    
                match &world.get::<BlocksTile>(*potential_target) {
                    Ok(_cs) => {
                        return false
                    }
                    Err(_e) => {}
                }
            }
        }

        return true
    }

    return false
}

pub fn get_path(map: &Map, from: Point, tp: Point) -> NavigationPath{
    let path = rltk::a_star_search(
        map.point_idx(from) as i32,
        map.point_idx(tp) as i32,
        map
    );

    return path;
}

pub fn autoexplore(gs: &mut State, entity: Entity){
    // TODO Check for adjacent enemies and attack them
    let entity_point: Point;
    
    // Use djikstras to find nearest unexplored tile
    let mut target = (Point::invalid_point(), std::f32::MAX); // tile_idx, distance
    let dijkstra_map: DijkstraMap;
    {
        let res = &gs.resources;
        let map: &mut Map = &mut res.get_mut::<Map>().unwrap();
        // let mut log = res.get_mut::<GameLog>().unwrap();

        let e_pos = if let Ok(pos) = gs.world.get::<Position>(entity){
            pos
        } else {
            dbg!("No position found");
            return;
        };

        let e_space = if let Ok(space) = gs.world.get::<SpatialKnowledge>(entity) {
            space
        } else {
            dbg!("Entity doesn't have a concept of space");
            return;
        };

        let e_idx = map.point_idx(e_pos.any_point());

        entity_point = e_pos.any_point();
        let starts: Vec<usize> = e_pos.idxes(map);
        dijkstra_map = rltk::DijkstraMap::new(map.width, map.height, &starts, map, 800.0);
        for (i, tile) in map.tiles.iter().enumerate() {
            if *tile != TileType::Wall && !e_space.tiles.contains_key(&i) {
                let distance_to_start = dijkstra_map.map[i];

                if distance_to_start < target.1 {
                    target = (map.idx_point(i), distance_to_start)
                }
            }
        }

        if target.1 == std::f32::MAX {
            // log.messages.push(format!("No tiles left to explore"));
            return;
        }

        // log.messages.push(format!("Closest unexplored tile is {} steps away", target.1));

        map.dijkstra_map = dijkstra_map.map.clone();

        // We have a target tile. Now follow the path up the chain
        let t = dijkstra_backtrace(dijkstra_map, map, e_idx, map.point_idx(target.0));
        target = (map.idx_point(t), 1.0);
    }
    
    // Send a move command
    let dx: i32;
    let dy: i32;
    {
        let res = &gs.resources;
        let map = res.get::<Map>().unwrap();
        // let targetx = map.idx_xy(target.0).0;
        // let targety = map.idx_xy(target.0).1;
        dx = target.0.x - entity_point.x;
        // if dx != 0 { dx = dx/dx.abs(); }
        dy = target.0.y - entity_point.y;
        // if dy != 0 { dy = dy / dy.abs(); }
    }
    
    // let t = map.
    try_move_entity(entity, Point { x: dx, y: dy }, gs);
}