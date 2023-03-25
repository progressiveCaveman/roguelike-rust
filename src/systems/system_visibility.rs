use rltk;
use rltk::{Point};
use shipyard::{View, ViewMut, UniqueView, IntoIter, IntoWithId, Get};
use crate::map::{Map};
use crate::components::{Position, Viewshed, SpatialKnowledge};

pub fn run_visibility_system(map: UniqueView<Map>, vpos: View<Position>, vvs: ViewMut<Viewshed>, vspace: View<SpatialKnowledge>) {
    for (id, (pos, vs)) in (&vpos, &vvs).iter().with_id() {
        // if vs.dirty {
            let pos = pos.ps.first().unwrap();

            vs.dirty = false;
            vs.visible_tiles = rltk::field_of_view(Point::new(pos.x, pos.y), vs.range, &*map);
            vs.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

            if let Ok(space) = vspace.get(id) {
                for vis in vs.visible_tiles.iter() {
                    let idx = map.xy_idx(vis.x, vis.y);
                    space.tiles.insert(idx, (map.tiles[idx], map.tile_content[idx].clone()));
                }
            }
        // }
    }
}


// pub fn run_visibility_system(world: &mut World, res: &mut Resources) {
//     let map: &mut Map = &mut res.get_mut::<Map>().unwrap();

//     for (_id, (pos, vs, space)) in world.query_mut::<(&Position, &mut Viewshed, Option<&mut SpatialKnowledge>)>() {
//         // if vs.dirty {
//             let pos = pos.ps.first().unwrap();

//             vs.dirty = false;
//             vs.visible_tiles = rltk::field_of_view(Point::new(pos.x, pos.y), vs.range, &*map);
//             vs.visible_tiles.retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

//             if let Some(space) = space {
//                 for vis in vs.visible_tiles.iter() {
//                     let idx = map.xy_idx(vis.x, vis.y);
//                     space.tiles.insert(idx, (map.tiles[idx], map.tile_content[idx].clone()));
//                 }
//             }
//         // }
//     }
// }
