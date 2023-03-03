use hecs::{Entity, World};
use resources::Resources;
use rltk::Point;

use crate::components::{SpatialKnowledge, Position};

#[derive(Clone, Debug)]
enum Task {
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
    DepositItem,
    Attack
}

#[derive(Clone, Debug)]
pub struct Intent {
    task: Task,
    target: Target,
    action_score: f32,
    turn: i32, // turn this intent originated
}

////////////////////////////////////////////////////

pub struct AI {
    entity: Entity
}

impl AI {
    fn example_action(&self, world: &World, res: &Resources) {

        // get my info
        // if let Ok(space) =  world.get_mut::<SpatialKnowledge>(self.entity) {
        let space =  world.get_mut::<SpatialKnowledge>(self.entity);
        let pos =  world.get_mut::<Position>(self.entity);
        let inventory =  world.get_mut::<>(self.entity);



        // if let Ok(space) =  world.get_mut::<SpatialKnowledge>(self.entity) {

        // let is =  world.get_mut::<(Position, SpatialKnowledge)>(self.entity);

        // get locations of potential targets

        // for (id, (wants_attack)) in &mut world.query::<(&WantsToAttack)>() {
        // }

        let potential_actions:Vec<Action> = vec!();

        // for each tree found
        potential_actions.push(Action {
            name: "go to tree".to_string(),
            cons: vec!(
                Consideration::new(
                    "Distance".to_string(), 
                    Inputs::distance(world, f, t),
                    ConsiderationParam { 
                        t: todo!(), 
                        m: 0.0, 
                        k: 0.0, 
                        c: 0.0, 
                        b: 0.0 
                    }
                ),
                Consideration::new(
                    "wood in stockpile".to_string(), 
                    Inputs::item_stockpile_count(world, stock, item_type),
                    ConsiderationParam { 
                        t: todo!(), 
                        m: 0.0, 
                        k: 0.0, 
                        c: 0.0, 
                        b: 0.0 
                    }
                )
            ),
            priority: 1.0,
        });

        // for each tree found
        potential_actions.push(Action {
            name: "chop tree".to_string(),
            cons: vec!(
                Consideration::new(
                    "Distance".to_string(), 
                    Inputs::distance(world, f, t),
                    ConsiderationParam { 
                        t: todo!(), 
                        m: 0.0, 
                        k: 0.0, 
                        c: 0.0, 
                        b: 0.0 
                    }
                ),
                Consideration::new(
                    "wood in stockpile".to_string(), 
                    Inputs::item_stockpile_count(world, stock, item_type),
                    ConsiderationParam { 
                        t: todo!(), 
                        m: 0.0, 
                        k: 0.0, 
                        c: 0.0, 
                        b: 0.0 
                    }
                )
            ),
            priority: 1.0,
        });

        // for each wood found
        potential_actions.push(Action {
            name: "pick up wood".to_string(),
            cons: vec!(
                Consideration::new(
                    "Distance".to_string(), 
                    Inputs::distance(world, f, t),
                    ConsiderationParam { 
                        t: todo!(), 
                        m: 0.0, 
                        k: 0.0, 
                        c: 0.0, 
                        b: 0.0 
                    }
                ),
                Consideration::new(
                    "wood in stockpile".to_string(), 
                    Inputs::item_stockpile_count(world, stock, item_type),
                    ConsiderationParam { 
                        t: todo!(), 
                        m: 0.0, 
                        k: 0.0, 
                        c: 0.0, 
                        b: 0.0 
                    }
                )
            ),
            priority: 1.0,
        });

        // if wood in inventory
        potential_actions.push(Action {
            name: "deliver wood".to_string(),
            cons: vec!(
                Consideration::new(
                    "Distance".to_string(), 
                    Inputs::distance(world, f, t),
                    ConsiderationParam { 
                        t: todo!(), 
                        m: 0.0, 
                        k: 0.0, 
                        c: 0.0, 
                        b: 0.0 
                    }
                ),
                Consideration::new(
                    "wood in stockpile".to_string(), 
                    Inputs::item_stockpile_count(world, stock, item_type),
                    ConsiderationParam { 
                        t: todo!(), 
                        m: 0.0, 
                        k: 0.0, 
                        c: 0.0, 
                        b: 0.0 
                    }
                )
            ),
            priority: 1.0,
        });

        // wander action
        potential_actions.push(Action {
            name: "wander".to_string(),
            cons: vec!(
                Consideration::new(
                    "baseline".to_string(), 
                    1.0,
                    ConsiderationParam::new_const(0.3)
                ),
            ),
            priority: 1.0,
        });

        let best_action = AI::evaluate_actions(potential_actions);
        best_action.perform_action(world);
    }

    fn evaluate_actions(actions: Vec<Action>) -> Action {
        let mut scores: Vec<f32> = vec!();
        let mut best_i = 0;
        let mut best_score = 0.;
        for i in 0..actions.len() {
            let action = actions[i];
            let score = action.get_action_score();

            scores.push(action.get_action_score());

            if score > best_score {
                best_score = score;
                best_i = i;
            }
        }

        actions[best_i]
    }
}

#[derive(Clone, Debug)]
pub struct Action {
    name: String,
    cons: Vec<Consideration>,
    priority: f32,
    action: (Entity, Intent) // the intent to attach 
}

impl Action {
    pub fn get_action_score(&self) -> f32 {
        // get average of all consideration scores
        let mut scores: Vec<f32> = vec!();
        for c in self.cons {
            scores.push(c.get_score());
        }

        let ave = average(&scores);

        // multiply by priorities
        ave * self.priority
    }

    pub fn perform_action(&self, world: &World) {
        // this is mostly apply intents I guess?
    }
}

#[derive(Clone, Debug)]
pub struct Consideration {
    name: String,
    input: f32,
    params: ConsiderationParam
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
        let t = self.params.t;
        let m = self.params.m;
        let k = self.params.k;
        let c = self.params.c;
        let b = self.params.b;

        let mut score = 0.;

        score = match t {
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
        };

        return score.clamp(0., 1.);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConsiderationParam {
    t: ResponseCurveType, 
    m: f32, 
    k: f32, 
    c: f32, 
    b: f32
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

#[derive(Clone, Debug, PartialEq)]
enum ResponseCurveType {
    Const,
    Linear,
    Quadratic,
    Logistic
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
    pub fn distance(world: &World, f: Target, t: Target) -> f32 {
        0.
    }

    pub fn item_stockpile_count(world: &World, stock: Target, item_type: Target) -> f32 {
        // tood fix item type
        0.
    }

    pub fn has_item(world: &World, f: Target, item_type: Target) -> f32 {
        0.
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

 #[derive(Clone, Debug)]
 enum Target {
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


pub fn average(numbers: &[f32]) -> f32 {
    let sum: f32 = numbers.iter().sum();
    let count = numbers.len() as f32;
    sum / count
}