use hecs::{Entity};
use rltk::{Point, DijkstraMap};

use crate::{State, utils::{InvalidPoint, dijkstra_backtrace}, map::{Map, TileType}, components::{Position, SpatialKnowledge}, movement::try_move_entity};

use super::EffectSpawner;

pub fn autoexplore(gs: &mut State, effect: &EffectSpawner, _: Entity){
    let res = &gs.resources;
    
    if let Some(entity) = effect.creator {
        // TODO Check for adjacent enemies and attack them
        let entity_point: Point;
        
        // Use djikstras to find nearest unexplored tile
        let mut target = (Point::invalid_point(), std::f32::MAX); // tile_idx, distance
        let dijkstra_map: DijkstraMap;
        {
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
            dx = target.0.x - entity_point.x;
            dy = target.0.y - entity_point.y;
        }
        
        try_move_entity(entity, Point { x: dx, y: dy }, gs);
    }
}