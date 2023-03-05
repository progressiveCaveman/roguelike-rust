use rltk::{DijkstraMap, BaseMap, Point, RGBA};

use crate::map::Map;

/// returns the point adjacent to origin that will lead to target
pub fn dijkstra_backtrace(dijkstra: DijkstraMap, map: &mut Map, origin: usize, mut target: usize) -> usize{
    for _ in 0..1000 {
        let neighbor_indices = map.get_available_exits(target);

        for &i in neighbor_indices.iter() {
            if i.0 == origin {
                return target;
            }

            if dijkstra.map[i.0] < dijkstra.map[target]{
                target = i.0;
            }
        }
    }

    target
}

pub trait Scale {
    fn scale(&mut self, amount: f32);
}

impl Scale for RGBA {
    fn scale(&mut self, amount: f32) {
        self.r *= amount;
        self.g *= amount;
        self.b *= amount;
    }
}

pub trait InvalidPoint {
    fn invalid_point() -> Point;
}

impl InvalidPoint for Point {
    fn invalid_point() -> Point {
        Point { x: 0, y: 0 }
    }
}