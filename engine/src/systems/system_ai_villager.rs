use crate::ai::decisions::{Action, Intent, Target, Task, AI};
use crate::ai::labors;
use crate::components::{DijkstraMapToMe, Position, Actor, ActorType};
use crate::effects::{add_effect, EffectType};
use crate::map::{Map, TileType};
use crate::utils::{get_neighbors, get_path, Turn};
use rltk;
use rltk::{BaseMap, Point};
use shipyard::{
    AddComponent, AllStoragesView, EntityId, Get, IntoIter, IntoWithId, UniqueView, View, ViewMut,
};

pub fn run_villager_ai_system(store: AllStoragesView) {
    update_decisions(&store);

    let map = store.borrow::<UniqueView<Map>>().unwrap();

    let mut to_move_from_to: Vec<(EntityId, Point, Point)> = vec![];
    let mut to_fish: Vec<(EntityId, Point)> = vec![];

    store.run(|vactor: View<Actor>, vpos: View<Position>, vintent: View<Intent>, vdijkstra: View<DijkstraMapToMe>| {
        for (id, (actor, pos, intent)) in (&vactor, &vpos, &vintent).iter().with_id() {
            if actor.atype != ActorType::Villager { // only run for villagers
                continue;
            }

            if actor.atype != ActorType::Villager { // only run for villagers
                dbg!("wat");
            }

            //world.query::<(&Villager, &mut Position, &mut Intent)>().iter() {
            match intent.task {
                Task::Fish => {
                    to_fish.push((id, pos.ps[0]));
                }
                Task::Explore => add_effect(Some(id), EffectType::Explore {}),
                Task::ExchangeInfo => todo!(),
                Task::MoveTo => {
                    if let Target::ENTITY(target) = intent.target[0] {
                        if let Ok(target_pos) = vpos.get(target) {
                            //world.get::<Position>(target) {
                            if let Ok(dijkstra) = vdijkstra.get(target) {
                                //world.get::<DijkstraMapToMe>(target) {
                                let my_idx = map.point_idx(pos.ps[0]);
                                let neighbor_indices = map.get_available_exits(my_idx);

                                let mut tidx: i32 = -1;
                                for &i in neighbor_indices.iter() {
                                    if tidx == -1
                                        || dijkstra.map.map[i.0]
                                            < dijkstra.map.map[tidx as usize]
                                    {
                                        tidx = i.0 as i32;
                                    }
                                }

                                to_move_from_to.push((
                                    id,
                                    pos.ps[0],
                                    map.idx_point(tidx as usize),
                                ));
                            } else {
                                to_move_from_to.push((id, pos.ps[0], target_pos.ps[0]));
                            }
                        }
                    } else if let Target::LOCATION(loc) = intent.target[0] {
                        to_move_from_to.push((id, pos.ps[0], loc));
                    }
                }
                Task::Destroy => {}
                Task::PickUpItem => {}
                Task::DropItem => todo!(),
                Task::UseItem => todo!(),
                Task::EquipItem => todo!(),
                Task::UnequipItem => todo!(),
                Task::UseWorkshop => todo!(),
                Task::DepositItemToInventory => {}
                Task::Attack => todo!(),
                Task::Idle => {},
            }
        }
    });

    for (e, from, to) in to_move_from_to {
        let map = store.borrow::<UniqueView<Map>>().unwrap();
        let path = get_path(&map, from, to);

        if path.success && path.steps.len() > 1 {
            // movement::try_move_entity(e, point_diff(from, p), gs);
            add_effect(
                Some(e),
                EffectType::Move {
                    tile_idx: path.steps[1],
                },
            );
        }
    }

    for (e, p) in to_fish {
        let n = get_neighbors(p);
        let adj_water: Vec<&Point> = n
            .iter()
            .filter(|p| {
                let idx = map.point_idx(**p);
                map.tiles[idx] == TileType::Water
            })
            .collect();

        for p in adj_water.iter() {
            let idx = map.point_idx(**p);
            for te in &map.tile_content[idx] {
                let vactor = store.borrow::<View<Actor>>().unwrap();
                if let Ok(actor) = vactor.get(*te) {
                    if actor.atype == ActorType::Fish {
                        //found a target
                        add_effect(Some(e), EffectType::PickUp { entity: *te });
                        break;
                    }
                }
            }
        }
    }
}

fn update_decisions(store: &AllStoragesView) {
    let mut wants_intent: Vec<(EntityId, Intent)> = vec![];
    let mut get_actions: Vec<EntityId> = vec![];

    let turn = store.borrow::<UniqueView<Turn>>().unwrap();

    let vactor = store.borrow::<View<Actor>>().unwrap();
    let mut vintent = store.borrow::<ViewMut<Intent>>().unwrap();

    for (id, _v) in (&vactor).iter().with_id().filter(|res| res.1.atype == ActorType::Villager) {
        // if we have a fresh intent, skip
        if let Ok(intent) = vintent.get(id) {
            if intent.turn.0 + 5 < turn.0 {
                continue;
            }
        }

        get_actions.push(id);
    }

    for id in get_actions {
        let mut potential_actions: Vec<Action> = vec![];

        potential_actions.append(&mut labors::get_gather_wood_actions(&store, id));
        potential_actions.append(&mut labors::get_gather_fish_actions(&store, id));

        let best = AI::choose_action(potential_actions).intent.clone();
        // dbg!(best.clone());
        wants_intent.push((id, best));
    }

    // let mut vintent = store.borrow::<ViewMut<Intent>>().unwrap();

    for (id, intent) in wants_intent {
        vintent.add_component_unchecked(id, intent);
    }
}
