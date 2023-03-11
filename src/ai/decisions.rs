use hecs::{Entity, World};
use resources::Resources;
use rltk::{Point, BaseMap};

use crate::{map::Map, components::{Position, ItemType, Inventory, Item}};

pub struct AI {
}

impl AI {
    pub fn choose_action(actions: Vec<Action>) -> Intent {
        let mut best_action: &Action = &actions[0];
        let mut best_score = 0.;

        for i in 0..actions.len() {
            let action = &actions[i];
            let score = action.get_action_score();

            // println!("Action: {}, score: {}", action.name, score);

            if score > best_score {
                best_score = score;
                best_action = action;
            }
        }

        best_action.intent.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Action {
    pub cons: Vec<Consideration>,
    pub priority: f32,
    pub intent: Intent,
}

impl Action {
    pub fn get_action_score(&self) -> f32 {
        // get average of all consideration scores
        let mut scores: Vec<f32> = vec!();
        for c in self.cons.iter() {
            let s = c.get_score();

            if s == 0. { 
                return 0.
            }

            scores.push(s);
        }

        let ave = average(&scores);

        // multiply by priorities
        ave * self.priority
    }
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Task {
    Fish,
    Explore,
    ExchangeInfo,
    MoveTo,
    Destroy,
    PickUpItem,
    DropItem,
    UseItem,
    EquipItem,
    UnequipItem,
    UseWorkshop,
    DepositItemToInventory,
    Attack
}

#[derive(Clone, Debug)]
pub struct Intent {
    pub name: String,
    pub task: Task,
    pub target: Vec<Target>, // most tasks have one target, more targets are specified in name, ie `DepositItemToInventory` expects [item, inventory]
    pub turn: i32, // turn this intent originated
}

#[derive(Clone, Debug)]
pub struct Consideration {
    pub name: String,
    pub input: f32,
    pub params: ConsiderationParam
}

impl Consideration {
    pub fn new(name: String, input: f32, params: ConsiderationParam) -> Consideration{
        Consideration { 
            name: name,
            input: input,
            params: params,
        }
    }

    fn get_score(&self) -> f32 {
        let t = &self.params.t;
        let m = self.params.m;
        let k = self.params.k;
        let c = self.params.c;
        let b = self.params.b;

        let score = match t {
            ResponseCurveType::Const => {
                m * self.input
            },
            ResponseCurveType::Quadratic | ResponseCurveType::Linear => {
               m * (self.input - c).powf(k) + b
            },
            ResponseCurveType::Logistic => {
                let e = std::f64::consts::E as f32;
                k * 1./(1.+(1000.*e*m).powf(-1. * self.input +c)) + b
            },
            ResponseCurveType::GreaterThan => {
                if self.input > m {
                    1.
                }else{
                    0.
                }
            },
            ResponseCurveType::LessThan => {
                if self.input < m {
                    1.
                }else{
                    0.
                }
            },
        };

        return score.clamp(0., 1.);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConsiderationParam {
    pub t: ResponseCurveType, 
    pub m: f32, 
    pub k: f32, 
    pub c: f32, 
    pub b: f32
}

impl ConsiderationParam {
    pub fn new_const(v: f32) -> ConsiderationParam {
        ConsiderationParam { 
            t: ResponseCurveType::Const, 
            m: v, 
            k: 0., 
            c: 0., 
            b: 0. 
        }
    }
}

/// for types Const, GreaterThan, and LessThan only m is considered
#[derive(Clone, Debug, PartialEq)]
pub enum ResponseCurveType {
    Const,
    GreaterThan,
    LessThan,
    Linear,
    Quadratic,
    Logistic,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InputType {
    // MyHealth,
    // MySpeed,
    DistanceToEntity,
    DistanceToComponentType,
    ItemStockpileCount,
    ItemRange,
    HasItem
}

pub struct Inputs {

}

impl Inputs {
    pub fn distance(world: &World, res: &Resources, f: Target, t: Target) -> f32 {
        let map: &mut Map = &mut res.get_mut::<Map>().unwrap();
        
        let idx1 = match f {
            Target::LOCATION(l) => vec!(map.xy_idx(l.x, l.y)),
            Target::ENTITY(e) => {
                if let Ok(p) = world.get::<Position>(e){
                    p.idxes(map)
                }else{
                    vec!(0)
                }
            },
        };

        let idx2 = match t {
            Target::LOCATION(l) => vec!(map.xy_idx(l.x, l.y)),
            Target::ENTITY(e) => {
                if let Ok(p) = world.get::<Position>(e){
                    p.idxes(map)
                }else{
                    vec!(0)
                }
            },
        };

        let mut min = f32::MAX;
        for i1 in idx1.iter() {
            for i2 in idx2.iter() {
                let dist = map.get_pathing_distance(*i1, *i2);
                if dist < min {
                    min = dist;
                }
            }
        }

        min
    }

    pub fn inventory_count(world: &World, holder: Entity, item_type: ItemType) -> f32 {
        let mut to_count_type: Vec<Entity> = vec![];
        if let Ok(mut inv) = world.get_mut::<Inventory>(holder) {
            to_count_type.append(&mut inv.items);
        }

        let mut count = 0;
        for e in to_count_type {
            if let Ok(item) = world.get::<Item>(e) {
                if item.typ == item_type {
                    count += 1;
                }
            }
        }

        return count as f32;
    }
}

/*
Use:
    let t = Target::from(point);

    match t {
        Target::LOCATION(value) => ,
        Target::ENTITY(value) => ,
    }
 */

 #[derive(Clone, Debug, Copy)]
 pub enum Target {
    LOCATION(Point),
    ENTITY(Entity),
}

impl From<Point> for Target {
    fn from(n: Point) -> Self {
        Target::LOCATION(n)
    }
}

impl From<Entity> for Target {
    fn from(n: Entity) -> Self {
        Target::ENTITY(n)
    }
}

impl Target {
    pub fn get_point(&self, world: &World) -> Point {
        match self {
            Target::LOCATION(loc) => *loc,
            Target::ENTITY(entity) => {
                if let Ok(pos) = world.get::<Position>(*entity) {
                    pos.ps[0]
                } else {
                    // dbg!("ERROR: Target::ENTITY position not found");
                    Point { x: -1, y: -1 }
                }
            },
        }
    }
}

pub fn average(numbers: &[f32]) -> f32 {
    let sum: f32 = numbers.iter().sum();
    let count = numbers.len() as f32;
    sum / count
}