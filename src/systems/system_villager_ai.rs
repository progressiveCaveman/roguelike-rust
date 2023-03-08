use hecs::*;
use rltk;
use rltk::Point;
use crate::ai::labors::get_wood_gathering_actions;
use crate::{State, movement};
use crate::ai::decisions::{Action, Consideration, ConsiderationParam, AI, Inputs, Target, Intent, Task, ResponseCurveType};
use crate::{RunState};
use crate::components::{Position, Villager, SpatialKnowledge, Inventory, Tree, Item, ItemType, LumberMill};

pub fn run_villager_ai_system(gs: &mut State) {
    
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
                    if let Target::ENTITY(target) = intent.target[0] {
                        if let Ok(target_pos) = world.get::<Position>(target) {
                            to_move_from_to.push((id, pos.ps[0], target_pos.ps[0]));
                        }
                    } else if let Target::LOCATION(loc) = intent.target[0] {
                        to_move_from_to.push((id, pos.ps[0], loc));
                    }
                },
                Task::Destroy => {

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
                Task::DepositItemToInventory => {
                    
                },
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

    let world = &gs.world;
    let res = &gs.resources;
    let turn = res.get::<i32>().unwrap();

    let mut wants_intent: Vec<(Entity, Intent)> = vec![];


    for (id, (_v, pos, space, inv, intent)) in world.query::<(&Villager, &Position, &SpatialKnowledge, &Inventory, Option<&Intent>)>().iter() {

        // if we have a fresh intent, skip
        if let Some(intent) = intent {
            if intent.turn + 5 < *turn {
                continue;
            }
        }
        
        let mut potential_actions:Vec<Action> = vec!();

        potential_actions.append(&mut get_wood_gathering_actions(gs, id, pos, space, inv));

        let best = AI::choose_action(potential_actions);
        // dbg!(best.clone());
        wants_intent.push((best.0, Intent { task: best.1, target: best.2, turn: *turn }));
    }

    for (id, intent) in wants_intent {
        let world = &mut gs.world;
        let _r = world.insert_one(id, intent);
    }
}