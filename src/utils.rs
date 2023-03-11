use rltk::{DijkstraMap, BaseMap, Point, RGBA};

use crate::map::Map;

/// returns the point adjacent to origin that will lead to target
pub fn dijkstra_backtrace(dijkstra: DijkstraMap, map: &mut Map, origin: usize, mut target: usize) -> usize{
    dbg!("dijkstra_backtrace");
    for _ in 0..1000 {
        dbg!("How many times does this run?");

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

pub fn get_neighbors(point: Point) -> Vec<Point> {
    vec![
        Point { x: point.x - 1, y: point.y - 1 },
        Point { x: point.x - 1, y: point.y },
        Point { x: point.x - 1, y: point.y + 1 },
        Point { x: point.x, y: point.y - 1 },
        Point { x: point.x, y: point.y + 1 },
        Point { x: point.x + 1, y: point.y - 1 },
        Point { x: point.x + 1, y: point.y },
        Point { x: point.x + 1, y: point.y + 1},
    ]
}

pub trait Scale {
    fn scaled(&mut self, amount: f32) -> RGBA;
}

impl Scale for RGBA {
    fn scaled(&mut self, amount: f32) -> RGBA {
        RGBA {
            r: self.r * amount,
            g: self.g * amount,
            b: self.b * amount,
            a: self.a * amount,
        }
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