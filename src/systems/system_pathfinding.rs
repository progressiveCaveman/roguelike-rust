use resources::Resources;
use crate::map::Map;
use crate::components::{Position, DijkstraMapToMe};

pub fn run_pathfinding_system(world: &mut World, res: &mut Resources) {
    let map: &mut Map = &mut res.get_mut::<Map>().unwrap();

    for (_, (pos, dijkstra)) in world.query_mut::<(&Position, &mut DijkstraMapToMe)>() {

        let mut starts: Vec<usize> = vec![];
        for pos in pos.ps.iter() {
            starts.push(map.point_idx(*pos));
        }

        dijkstra.map = rltk::DijkstraMap::new(map.width, map.height, &starts, map, 100.0);
    }
}
