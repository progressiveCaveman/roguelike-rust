use rltk::{Point, NavigationPath};
use shipyard::{EntityId, World};

use crate::utils::{point_plus, WorldGet};
use crate::{State, GameMode};
use crate::map::{Map, TileType};
use crate::components::{Position, Player, Viewshed, CombatStats, WantsToAttack, BlocksTile, Locomotive, LocomotionType};

/// dp is delta
pub fn try_move_entity(entity: EntityId, dp: Point, gs: &mut State) {
    let mut map = gs.get_map();//gs.resources.get_mut::<Map>().unwrap();
    let mode = gs.get_game_mode();//gs.resources.get::<GameMode>().unwrap();
    let mut needs_wants_to_attack: Option<(EntityId, WantsToAttack)> = None;

    // if tp.x < 0 || tp.y < 0 || tp.x >= map.width || tp.y >= map.height { return; }

    if let Ok(mut pos) = gs.world.get::<Position>(entity) {
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
                if let Ok(mut vs) = gs.world.get::<Viewshed>(entity) {
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
                match &gs.world.get::<Player>(entity) {
                    Err(_e) => {},
                    Ok(_player) => {
                        let mut ppos = gs.get_player_pos();//gs.resources.get_mut::<rltk::Point>().unwrap();
                        ppos.0.x = pos.ps[0].x;
                        ppos.0.y = pos.ps[0].y;
                    }
                }

                return;
            }
        // }
    }

    if let Some((e, c)) = needs_wants_to_attack {
        let _res = gs.world.add_component(e, c);
    }
}

// checks for entities with combat stats on block
pub fn get_target(world: &World, map: &Map, entity: EntityId, pos: &Position, dp: Point) -> Option<EntityId> {

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

pub fn can_move(world: &World, map: &Map, entity: EntityId, pos: &Position, dp: Point) -> bool {
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
