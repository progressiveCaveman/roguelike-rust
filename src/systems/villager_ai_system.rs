use hecs::*;
use rltk;
use rltk::Point;
use crate::{State, movement};
use crate::ai::decisions::{Action, Consideration, ConsiderationParam, AI, Inputs, Target, Intent, Task, ResponseCurveType};
use crate::{RunState};
use crate::components::{Position, Villager, SpatialKnowledge, Inventory, Tree, Item, ItemType, LumberMill};

pub fn villager_ai(gs: &mut State) {

    {
        let res = &mut gs.resources;
        let runstate: &RunState = &res.get::<RunState>().unwrap();
        if *runstate != RunState::AiTurn { return; }
    }
    
    update_decisions(gs);

    let mut to_explore: Vec<Entity> = vec![];
    let mut to_move_from_to: Vec<(Entity, Point, Point)> = vec![];

    {
        let world = &mut gs.world;
        let res = &mut gs.resources;

        let runstate: &RunState = &res.get::<RunState>().unwrap();
        if *runstate != RunState::AiTurn { return; }


        for (id, (_, pos, space, inv, intent)) in world.query::<(&Villager, &mut Position, &mut SpatialKnowledge, &mut Inventory, &mut Intent)>().iter() {
            match intent.task {
                Task::Fish => todo!(),
                Task::Explore => {
                    println!("Exploring....");
                    to_explore.push(id);
                },
                Task::ExchangeInfo => todo!(),
                Task::MoveTo => {
                    if let Some(Target::ENTITY(target)) = intent.target {
                        if let Ok(target_pos) = world.get::<Position>(target) {
                            to_move_from_to.push((id, pos.ps[0], target_pos.ps[0]));
                        }
                    } else if let Some(Target::LOCATION(loc)) = intent.target {
                        to_move_from_to.push((id, pos.ps[0], loc));
                    }
                },
                Task::Destroy => {
                    dbg!("Destroy");
                },
                Task::PickUpItem => {
                    // if let Some(Target::ENTITY(t)) = intent.target {
                    //     world.insert_one(id, WantsToPickupItem{ collected_by: id, item: t }).unwrap();
                    // }
                },
                Task::DropItem => todo!(),
                Task::UseItem => todo!(),
                Task::EquipItem => todo!(),
                Task::UnequipItem => todo!(),
                Task::UseWorkshop => todo!(),
                Task::DepositItem => todo!(),
                Task::Attack => todo!(),
            }
        }
    }

    for e in to_explore {
        movement::autoexplore(gs, e);
        let world = &mut gs.world;
        let _res = world.remove_one::<Intent>(e);
    }

    for (e, from, to) in to_move_from_to {
        let mut dx = to.x - from.x;
        let mut dy = to.y - from.y;

        if dx != 0 { dx = dx / dx.abs(); }
        if dy != 0 { dy = dy / dy.abs(); }

        movement::try_move_entity(e, dx, dy, gs);
        let world = &mut gs.world;
        let _res = world.remove_one::<Intent>(e);
    }
}

fn update_decisions(gs: &mut State) {

    let world = &mut gs.world;
    let res = &mut gs.resources;
    let turn = res.get::<i32>().unwrap();

    let mut wants_intent: Vec<(Entity, Intent)> = vec![];


    for (id, (_v, pos, space, inv, intent)) in world.query::<(&Villager, &mut Position, &mut SpatialKnowledge, &mut Inventory, Option<&mut Intent>)>().iter() {

        // if we have a fresh intent, skip
        if let Some(intent) = intent {
            if intent.turn + 5 < *turn {
                continue;
            }
        }

        println!("update_decisions");

        let pos = pos.ps[0];

        let has_inventory_space = inv.capacity > inv.items.len() as i32;

        let mut logs_in_inv = 0;
        for e in inv.items.iter() {
            if let Ok(item) = world.get::<Item>(*e) {
                if item.typ == ItemType::Log {
                    logs_in_inv += 1;
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
                    lumber_mills.push(*e);
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
                    action: (id, Task::MoveTo, Some(Target::from(tree))),
                });

                potential_actions.push(Action {
                    name: "chop tree".to_string(),
                    cons: vec!(
                        Consideration::new(
                            "Distance".to_string(), 
                            Inputs::distance(world, res, Target::from(pos), Target::from(tree)),
                            ConsiderationParam { 
                                t: ResponseCurveType::Linear, 
                                m: -1.0, 
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
                    action: (id, Task::Destroy, Some(Target::from(tree))),
                });
            }
        }

        // for each wood found
        for log in logs {
            if has_inventory_space {
                potential_actions.push(Action {
                    name: "pick up wood".to_string(),
                    cons: vec!(
                        Consideration::new(
                            "Distance".to_string(), 
                            Inputs::distance(world, res, Target::from(pos), Target::from(log)),
                            ConsiderationParam { 
                                t: ResponseCurveType::Linear, 
                                m: -1.0 / 100.0, 
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
                    action: (id, Task::PickUpItem, Some(Target::from(log))),
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
                                m: -1.0 / 100.0, 
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
                    action: (id, Task::MoveTo, Some(Target::from(lm))),
                });

                potential_actions.push(Action {
                    name: "deposit logs at lumber mill".to_string(),
                    cons: vec!(
                        Consideration::new(
                            "Distance".to_string(), 
                            Inputs::distance(world, res, Target::from(pos), Target::from(lm)),
                            ConsiderationParam { 
                                t: ResponseCurveType::Linear, 
                                m: -1.0, 
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
                    priority: 2.0,
                    action: (id, Task::DepositItem, Some(Target::from(lm))),
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
            action: (id, Task::Explore, None),
        });

        let best = AI::choose_action(potential_actions);
        dbg!(best.clone());
        wants_intent.push((best.0, Intent { task: best.1, target: best.2, turn: *turn }));
    }

    for (id, intent) in wants_intent {
        let _r = world.insert_one(id, intent);
    }
}