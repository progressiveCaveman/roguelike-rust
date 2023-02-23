use std::cmp::{max, min};
use hecs::*;

use crate::{State, MAPWIDTH, MAPHEIGHT, GameMode};
use crate::map::{Map};
use crate::components::{Position, Player, Viewshed, CombatStats, WantsToAttack, Locomotive, BlocksTile};

pub fn try_move_entity(entity: Entity, dx: i32, dy: i32, gs: &mut State) {
    let map = gs.resources.get::<Map>().unwrap();
    let mode = gs.resources.get::<GameMode>().unwrap();
    let mut needs_wants_to_attack: Option<(Entity, WantsToAttack)> = None;

    if let Ok(mut pos) = gs.world.get_mut::<Position>(entity) {
        if let Ok(_loc) = &gs.world.get::<Locomotive>(entity) {

            // In Sim mode, player is basically just a camera object
            let mut is_camera = false;
            if let Ok(_) = &gs.world.get::<Player>(entity) {
                if *mode == GameMode::Sim { is_camera = true; }
            }

            // check for blockers
            let mut is_blocked = false;

            if !is_camera {
                // If there's anything attackable in path, attack it
                for pos in pos.ps.iter() {
                    let dest_idx = map.xy_idx(pos.x + dx, pos.y + dy);
                    for potential_target in map.tile_content[dest_idx].iter() {
                        if *potential_target == entity {
                            continue;
                        }

                        match &gs.world.get::<CombatStats>(*potential_target) {
                            Ok(_cs) => {
                                needs_wants_to_attack = Some((entity, WantsToAttack {target: *potential_target}));
                                break;
                            }
                            Err(_e) => {}
                        }
                    }
                }

                // This is ugly but it basically says if we have an intention to attack, don't try to move
                if let Some(_) = needs_wants_to_attack {
                    is_blocked = true
                }

                for pos in pos.ps.iter() {
                    if is_blocked { break; }

                    // check for tiles that block
                    let dest_idx = map.xy_idx(pos.x + dx, pos.y + dy);
                    if map.blocks_movement(dest_idx) { 
                        is_blocked = true;
                        break;
                    }

                    // Check for entities that block
                    for potential_blocker in map.tile_content[dest_idx].iter() {
                        if *potential_blocker == entity {
                            continue;
                        }

                        match &gs.world.get::<BlocksTile>(*potential_blocker) {
                            Ok(_cs) => {
                                is_blocked = true;
                                break;
                            }
                            Err(_e) => {}
                        }
                    }
                }
            }


            if is_camera || is_blocked == false {                
                if let Ok(mut vs) = gs.world.get_mut::<Viewshed>(entity) {
                    vs.dirty = true;
                }

                // for pos in pos.ps.iter_mut() {
                for i in 0..pos.ps.len() {
                    pos.ps[i].x = min(MAPWIDTH as i32 - 1, max(0, pos.ps[i].x + dx));
                    pos.ps[i].y = min(MAPHEIGHT as i32 - 1, max(0, pos.ps[i].y + dy));   
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
            }
        }
    }

    if let Some(v) = needs_wants_to_attack {
        let _res = gs.world.insert_one(v.0, v.1);
    }
}