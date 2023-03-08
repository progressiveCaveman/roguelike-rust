use hecs::*;
use resources::Resources;
use crate::map::Map;
use crate::components::{BlocksTile, Position};


pub fn map_indexing(world: &mut World, res: &mut Resources) {
    let map: &mut Map = &mut res.get_mut::<Map>().unwrap();

    map.set_blocked();
    map.clear_tile_content();

    for (id, (_bt, pos)) in world.query_mut::<(Option<&BlocksTile>, &Position)>() {
        for pos in pos.ps.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            if idx > map.tiles.len() { continue }
    
            if let Some(_bt) = _bt {
                map.blocked[idx] = true;
            }
    
            map.tile_content[idx].push(id);
        }
    }
}
