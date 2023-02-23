use hecs::*;
use resources::Resources;
use rltk;
use rltk::{Point};
use crate::{GameMode};
use crate::map::{Map};
use crate::components::{Position, Viewshed, Player};


pub fn visibility(world: &mut World, res: &mut Resources) {
    let map: &mut Map = &mut res.get_mut::<Map>().unwrap();
    let gamemode = *res.get::<GameMode>().unwrap();

    if gamemode == GameMode::Sim {
        return;
    }

    for (_id, (pos, vs, player)) in world.query_mut::<(&Position, &mut Viewshed, Option<&Player>)>() {
        if vs.dirty {
            let pos = pos.ps.first().unwrap();

            vs.dirty = false;
            vs.visible_tiles = rltk::field_of_view(Point::new(pos.x, pos.y), vs.range, &*map);
            vs.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

            if let Some(_player) = player {
                for t in map.visible_tiles.iter_mut() {
                    *t = false;
                }
                for vis in vs.visible_tiles.iter() {
                    let idx = map.xy_idx(vis.x, vis.y);
                    map.revealed_tiles[idx] = true;
                    map.visible_tiles[idx] = true;
                }
            }
        }
    }
}
