use rand::prelude::SliceRandom;
use hecs::*;
use rand::thread_rng;
use rltk::Point;
use crate::map::{TileType};
use crate::components::{Position, Fish};
use crate::utils::point_diff;
use crate::{movement, State};

// currently fish only move east
pub fn run_fish_ai(gs: &mut State) {    
    let mut to_try_move: Vec<(Entity, Point)> = vec![];
    
    for (id, (pos, _)) in gs.world.query::<(&Position, &Fish)>().iter() {        
        if pos.ps.len() == 1{
            // if at edge of map, remove fish

            let pos = pos.ps[0];
            to_try_move.push((id, pos));
        } else {
            dbg!("ERROR: multi-tile fish not supported");
        }
    }

    for (e, pos) in to_try_move {
        let mut potential_spaces = vec![
            Point {x: pos.x + 1, y: pos.y},
            Point {x: pos.x + 1, y: pos.y + 1},
            Point {x: pos.x + 1, y: pos.y - 1},
        ];

        potential_spaces.shuffle(&mut thread_rng());

        for ps in potential_spaces {
            let canmove = {
                let idx = gs.get_map().point_idx(ps);
                gs.get_map().tiles[idx] == TileType::Water
            };

            if canmove {
                movement::try_move_entity(e, point_diff(pos, ps), gs);
                break;
            }
        }
    }
}