use rand::prelude::SliceRandom;
use hecs::*;
use rand::thread_rng;
use resources::Resources;
use rltk::Point;
use crate::map::{Map, TileType};
use crate::components::{Position, Fish};

// currently only supports fish moving east
pub fn run_fish_ai(world: &mut World, res: &mut Resources) {    
    // let mut log = res.get_mut::<GameLog>().unwrap();
    let map = res.get::<Map>().unwrap();

    let mut to_move: Vec<(Entity, usize)> = vec![];

    for (id, (pos, fish)) in world.query_mut::<(&Position, &Fish)>() {        
        if pos.ps.len() == 1{
            // if at edge of map, remove fish

            let pos = pos.ps[0];
            let mut potential_spaces = vec![
                Point {x: pos.x + 1, y: pos.y},
                Point {x: pos.x + 1, y: pos.y + 1},
                Point {x: pos.x + 1, y: pos.y - 1},
            ];

            potential_spaces.shuffle(&mut thread_rng());

            for ps in potential_spaces {
                let idx = map.xy_idx(ps.x + 1, ps.y);
                if map.tiles[idx] == TileType::Water {
                    to_move.push((id, idx));
                    break;
                }
            }
        } else {
            dbg!("ERROR: multi-tile fish not supported");
        }
    }

    for (e, p) in to_move {
        if let Ok(mut pos) = world.get_mut::<Position>(e) { // todo run try_move here instead
            let point = map.idx_point(p);
            pos.ps[0] = point;
        }
    }
}