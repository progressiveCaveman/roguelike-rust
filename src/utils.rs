use rltk::{DijkstraMap, BaseMap, Point, RGBA};
use shipyard::{World, EntityId, ViewMut, Get, Unique};

use crate::map::Map;

/// returns the point adjacent to origin that will lead to target
pub fn dijkstra_backtrace(dijkstra: DijkstraMap, map: &mut Map, origin: usize, mut target: usize) -> usize{
    // dbg!("dijkstra_backtrace");
    for _ in 0..1000 {
        // dbg!("How many times does this run?");

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

// translates dir according to roguelike numpad convention - 1 is SW, 9 is NE
// pub fn get_movement(point: Point, dir: usize, movemod: i32) -> Point {
//     match dir {
//         1 => Point { x: point.x - movemod, y: point.y + movemod },
//         2 => Point { x: point.x, y: point.y + movemod },
//         3 => Point { x: point.x + movemod, y: point.y + movemod },
//         4 => Point { x: point.x - movemod, y: point.y },
//         6 => Point { x: point.x + movemod, y: point.y },
//         7 => Point { x: point.x - movemod, y: point.y - movemod },
//         8 => Point { x: point.x, y: point.y - movemod },
//         9 => Point { x: point.x + movemod, y: point.y - movemod },
//         _ => point
//     }
// }

// translates dir according to roguelike numpad convention - 1 is SW, 9 is NE
pub fn dir_to_point(dir: usize, dismod: i32) -> Point {
    match dir {
        1 => Point { x: -dismod, y: dismod },
        2 => Point { x: 0, y: dismod },
        3 => Point { x: dismod, y: dismod },
        4 => Point { x: -dismod, y: 0 },
        6 => Point { x: dismod, y: 0 },
        7 => Point { x: -dismod, y: -dismod },
        8 => Point { x: 0, y: -dismod },
        9 => Point { x: dismod, y: -dismod },
        _ => Point { x: 0, y: 0 }
    }
}

pub fn point_plus(p1: Point, p2: Point) -> Point {
    Point { x: p2.x + p1.x, y: p2.y + p1.y }
}

pub fn point_diff(p1: Point, p2: Point) -> Point {
    Point { x: p2.x - p1.x, y: p2.y - p1.y }
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
        Point { x: -1, y: -1 }
    }
}

#[derive(Clone, Debug, Unique)]
pub struct Turn(i32);

// pub trait WorldGet {
//     fn get<T: shipyard::Component + std::marker::Sync + std::marker::Send>(&self, entity: EntityId) -> Result<&T, shipyard::error::MissingComponent>;
// }

// impl WorldGet for World {
//     fn get<T: shipyard::Component + std::marker::Sync + std::marker::Send>(&self, entity: EntityId) -> Result<&T, shipyard::error::MissingComponent> {
//         if let Ok(s) = self.borrow::<ViewMut<T>>(){
//             return (&s).get(entity);
//         }

//         return Err(shipyard::error::MissingComponent {
//             id: entity,
//             name: "Missing Component",
//         });
//     }
// }