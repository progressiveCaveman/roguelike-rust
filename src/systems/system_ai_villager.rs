use hecs::*;
use rltk;
use rltk::{Point, BaseMap};
use crate::ai::labors::{get_wood_gathering_actions, get_fishing_actions};
use crate::effects::{add_effect, EffectType, Targets};
use crate::map::{Map, TileType};
use crate::utils::{get_neighbors, point_diff};
use crate::{State, movement};
use crate::ai::decisions::{Action, AI, Target, Intent, Task};
use crate::components::{Position, Villager, SpatialKnowledge, Inventory, DijkstraMapToMe, Fish};

pub fn run_villager_ai_system(gs: &mut State) {
    
    update_decisions(gs);

    let mut to_explore: Vec<Entity> = vec![];
    let mut to_move_from_to: Vec<(Entity, Point, Point)> = vec![];
    let mut to_fish: Vec<(Entity, Point)> = vec![];

    {
        let world = &mut gs.world;
        let res = &mut gs.resources;
        let map: &mut Map = &mut res.get_mut::<Map>().unwrap();

        for (id, (_, pos, intent)) in world.query::<(&Villager, &mut Position, &mut Intent)>().iter() {
            match intent.task {
                Task::Fish => {
                    to_fish.push((id, pos.ps[0]));
                },
                Task::Explore => {
                    // println!("Exploring....");
                    to_explore.push(id);
                },
                Task::ExchangeInfo => todo!(),
                Task::MoveTo => {
                    if let Target::ENTITY(target) = intent.target[0] {
                        if let Ok(target_pos) = world.get::<Position>(target) {
                            if let Ok(dijkstra) = world.get::<DijkstraMapToMe>(target) {
                                let my_idx = map.point_idx(pos.ps[0]);
                                let neighbor_indices = map.get_available_exits(my_idx);

                                let mut tidx:i32 = -1;
                                for &i in neighbor_indices.iter() {
                                    if tidx == -1 || dijkstra.map.map[i.0] < dijkstra.map.map[tidx as usize]{
                                        tidx = i.0 as i32;
                                    }
                                }

                                to_move_from_to.push((id, pos.ps[0], map.idx_point(tidx as usize)));
                            }else{
                                to_move_from_to.push((id, pos.ps[0], target_pos.ps[0]));
                            }
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
        add_effect(Some(e), EffectType::Explore {  }, Targets::Single { target: e })
    }

    for (e, from, to) in to_move_from_to {
        let path = movement::get_path(&gs.get_map(), from, to);

        if path.success && path.steps.len() > 1 {
            let p = gs.get_map().idx_point(path.steps[1]);
            movement::try_move_entity(e, point_diff(from, p), gs);
        }
    }

    for (e, p) in to_fish {
        let world = &mut gs.world;
        let res = &mut gs.resources;
        
        let map = &res.get::<Map>().unwrap();

        let n = get_neighbors(p);
        let adj_water: Vec<&Point> = n.iter().filter(|p| {
            let idx = map.point_idx(**p);
            map.tiles[idx] == TileType::Water
        }).collect();
        

        for p in adj_water.iter() {
            let idx = map.point_idx(**p);
            for te in &map.tile_content[idx] {
                if let Ok(_) = world.get::<Fish>(*te) {
                    //found a target
                    add_effect(
                        Some(e), 
                        EffectType::PickUp {}, 
                        Targets::Single { target: *te }
                    );
                    break;
                }
            }
        }
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
        potential_actions.append(&mut get_fishing_actions(gs, id, pos, space, inv));

        let best = AI::choose_action(potential_actions);
        // dbg!(best.clone());
        wants_intent.push((id, best));
    }

    for (id, intent) in wants_intent {
        let world = &mut gs.world;
        let _r = world.insert_one(id, intent);
    }
}