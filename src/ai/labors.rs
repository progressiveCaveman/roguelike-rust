use hecs::Entity;

use crate::{State, components::{Position, SpatialKnowledge, Inventory, ItemType, Item, Tree, LumberMill}};

use super::decisions::{Action, Consideration, Inputs, ConsiderationParam, Target, ResponseCurveType, Task};

pub fn get_wood_gathering_actions(gs: &State, id: Entity, pos: &Position, space: &SpatialKnowledge, inv: &Inventory) -> Vec<Action>{

    let world = &gs.world;
    let res = &gs.resources;

    let pos = pos.ps[0];

    let has_inventory_space = inv.capacity > inv.items.len() as i32;

    let mut logs_in_inv = 0;
    let mut inventory_log: Entity = id; // initialization is messy here but correct as long as logs_in_inv > 0
    for e in inv.items.iter() {
        if let Ok(item) = world.get::<Item>(*e) {
            if item.typ == ItemType::Log {
                logs_in_inv += 1;
                inventory_log = *e;
            }
        }
    }

    // populate all our info
    let mut trees: Vec<Entity> = vec![];
    let mut logs: Vec<Entity> = vec![];
    let mut lumber_mills: Vec<Entity> = vec![];
    for (_, entities) in space.tiles.values() {
        for e in entities.iter() {
            if let Ok(_) = world.get::<Tree>(*e) {
                trees.push(*e);
            }
            if let Ok(item) = world.get::<Item>(*e) {
                if item.typ == ItemType::Log {
                    logs.push(*e);
                }
            }
            if let Ok(_) = world.get::<LumberMill>(*e) {
                if !lumber_mills.contains(e) { //multitile
                    lumber_mills.push(*e);
                }
            }
        }
    }

    let mut potential_actions:Vec<Action> = vec!();

    // for each tree found
    for tree in trees{
        if has_inventory_space {
            potential_actions.push(Action {
                name: "go to tree".to_string(),
                cons: vec!(
                    Consideration::new(
                        "Distance".to_string(), 
                        Inputs::distance(world, res, Target::from(pos), Target::from(tree)),
                        ConsiderationParam { 
                            t: ResponseCurveType::Linear, 
                            m: -1.0 / 100.0, 
                            k: 1.0, 
                            c: 1.0, 
                            b: 1.0 
                        }
                    ),
                    // Consideration::new(
                    //     "wood in stockpile".to_string(), 
                    //     Inputs::item_stockpile_count(world, stock, item_type),
                    //     ConsiderationParam { 
                    //         t: todo!(), 
                    //         m: 0.0, 
                    //         k: 0.0, 
                    //         c: 0.0, 
                    //         b: 0.0 
                    //     }
                    // )
                ),
                priority: 1.0,
                action: (id, Task::MoveTo, vec!(Target::from(tree))),
            });

            potential_actions.push(Action {
                name: "chop tree".to_string(),
                cons: vec!(
                    Consideration::new(
                        "Distance".to_string(), 
                        Inputs::distance(world, res, Target::from(pos), Target::from(tree)),
                        ConsiderationParam { 
                            t: ResponseCurveType::LessThan, 
                            m: 2., 
                            k: 1.0, 
                            c: 1.0, 
                            b: 1.0 
                        }
                    ),
                    // Consideration::new(
                    //     "wood in stockpile".to_string(), 
                    //     Inputs::item_stockpile_count(world, stock, item_type),
                    //     ConsiderationParam { 
                    //         t: todo!(), 
                    //         m: 0.0, 
                    //         k: 0.0, 
                    //         c: 0.0, 
                    //         b: 0.0 
                    //     }
                    // )
                ),
                priority: 2.0,
                action: (id, Task::Destroy, vec!(Target::from(tree))),
            });
        }
    }

    // for each wood found
    for log in logs.iter() {
        if has_inventory_space {
            potential_actions.push(Action {
                name: "pick up wood".to_string(),
                cons: vec!(
                    Consideration::new(
                        "Distance".to_string(), 
                        Inputs::distance(world, res, Target::from(pos), Target::from(*log)),
                        ConsiderationParam { 
                            t: ResponseCurveType::LessThan, 
                            m: 2., 
                            k: 1.0, 
                            c: 0.0, 
                            b: 1.0 
                        }
                    ),
                    // Consideration::new(
                    //     "wood in stockpile".to_string(), 
                    //     Inputs::item_stockpile_count(world, stock, item_type),
                    //     ConsiderationParam { 
                    //         t: todo!(), 
                    //         m: 0.0, 
                    //         k: 0.0, 
                    //         c: 0.0, 
                    //         b: 0.0 
                    //     }
                    // )
                ),
                priority: 1.0,
                action: (id, Task::PickUpItem, vec!(Target::from(*log))),
            });
        }
    }

    // if wood in inventory
    // for each LumberMill
    for lm in lumber_mills {
        if logs_in_inv > 0 {
            potential_actions.push(Action {
                name: "move to lumber mill".to_string(),
                cons: vec!(
                    Consideration::new(
                        "Distance".to_string(), 
                        Inputs::distance(world, res, Target::from(pos), Target::from(lm)),
                        ConsiderationParam { 
                            t: ResponseCurveType::Linear, 
                            m: 1. - 1./20., 
                            k: 1.0, 
                            c: 1.0, 
                            b: 0.0 
                        }
                    ),
                    Consideration::new(
                        "logs in stockpile".to_string(), 
                        Inputs::inventory_count(world, lm, ItemType::Log),
                        ConsiderationParam { 
                            t: ResponseCurveType::Linear, 
                            m: -1. / 50.0, 
                            k: 1.0, 
                            c: 0.0, 
                            b: 1.0 
                        }
                    ),
                    Consideration::new(
                        "logs in iventory".to_string(), 
                        inv.count_type(world, ItemType::Log) as f32,
                        ConsiderationParam { 
                            t: ResponseCurveType::Linear, 
                            m: 1. / 5.0, 
                            k: 1.0, 
                            c: 0.0, 
                            b: 0.0 
                        }
                    )
                ),
                priority: 1.0,
                action: (id, Task::MoveTo, vec!(Target::from(lm))),
            });

            potential_actions.push(Action {
                name: "deposit logs at lumber mill".to_string(),
                cons: vec!(
                    Consideration::new(
                        "Distance to lm".to_string(), 
                        Inputs::distance(world, res, Target::from(pos), Target::from(lm)),
                        ConsiderationParam { 
                            t: ResponseCurveType::LessThan, 
                            m: 2., 
                            k: 2.0, 
                            c: 1.0, 
                            b: 0.0 
                        }
                    ),
                    Consideration::new(
                        "logs in stockpile".to_string(), 
                        Inputs::inventory_count(world, lm, ItemType::Log),
                        ConsiderationParam { 
                            t: ResponseCurveType::Linear, 
                            m: -1. / 50.0, 
                            k: 1.0, 
                            c: 0.0, 
                            b: 1.0 
                        }
                    ),
                    Consideration::new(
                        "logs in iventory".to_string(), 
                        inv.count_type(world, ItemType::Log) as f32,
                        ConsiderationParam { 
                            t: ResponseCurveType::Linear, 
                            m: 1. / 5.0, 
                            k: 1.0, 
                            c: 0.0, 
                            b: 0.0 
                        }
                    )
                ),
                priority: 2.0,
                action: (id, Task::DepositItemToInventory, vec!(Target::from(inventory_log), Target::from(lm))),
            });

        }
    }

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
        action: (id, Task::Explore, vec![]),
    });

    potential_actions
}