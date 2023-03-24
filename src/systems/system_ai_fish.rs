use rand::prelude::SliceRandom;
use rand::thread_rng;
use rltk::Point;
use shipyard::{EntityId, View, IntoIter, IntoWithId};
use crate::map::{TileType, Map};
use crate::components::{Position, Fish};
use crate::utils::point_diff;
use crate::{movement, State};

// currently fish only move east
pub fn run_fish_ai(gs: &mut State) {    
    let mut to_try_move: Vec<(EntityId, Point)> = vec![];
    let mut to_remove: Vec<EntityId> = vec![];

    {
        let map: &Map = &mut gs.resources.get::<Map>().unwrap();
        
        gs.world.run(|vpos: View<Position>, vfish: View<Fish>| {
            for (id, (pos, fish)) in (&vpos, &vfish).iter().with_id() {
                if pos.ps.len() == 1{
                    // if at edge of map, remove fish
    
                    let pos = pos.ps[0];
    
                    if pos.x >= map.width - 2 {
                        to_remove.push(id);
                    } else {
                        to_try_move.push((id, pos));
                    }
                } else {
                    dbg!("ERROR: multi-tile fish not supported");
                }
            }
        });
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

    for e in to_remove.iter() {
        gs.world.delete_entity(*e);
    }
}