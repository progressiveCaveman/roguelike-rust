use shipyard::{ViewMut, View, UniqueView, IntoIter, IntoWithId};
use crate::map::Map;
use crate::components::{Position, DijkstraMapToMe};

pub fn run_pathfinding_system(map: UniqueView<Map> , vpos: View<Position>, vmaps: ViewMut<DijkstraMapToMe>) {
    for (id, (pos, dijkstra)) in (&vpos, &vmaps).iter().with_id() {
        let mut starts: Vec<usize> = vec![];
        for pos in pos.ps.iter() {
            starts.push(map.point_idx(*pos));
        }

        dijkstra.map = rltk::DijkstraMap::new(map.width, map.height, &starts, &*map, 100.0);
    }
}