use std::sync::Mutex;
use std::collections::VecDeque;

mod damage;
pub use damage::inflict_damage;

mod delete;

mod confusion;
pub use confusion::inflict_confusion;

mod fire;
pub use fire::inflict_fire;

mod heal;

mod inventory;
pub use inventory::pick_up;

mod movement;

use shipyard::{EntityId, UniqueView, View, Get, World};

use crate::{State, map::Map, components::Position};

lazy_static! {
    pub static ref EFFECT_QUEUE : Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

#[derive(Clone)]
pub enum EffectType { 
    Damage { amount : i32, target: Targets},
    Confusion { turns: i32, target: Targets },
    Fire { turns: i32, target: Targets },
    PickUp { entity: EntityId },
    Drop { entity: EntityId },
    Explore { },
    Heal { amount: i32, target: Targets},
    Move { tile_idx: usize },
    MoveOrAttack { tile_idx: usize },
    Wait {},
    Delete { entity: EntityId },
}

#[derive(Clone)]
pub enum Targets {
    Tile { tile_idx: usize},
    Tiles { tiles: Vec<usize> },
    Single { target: EntityId },
    Area { target: Vec<EntityId> },
}

#[derive(Clone)]
pub struct EffectSpawner {
    pub creator : Option<EntityId>,
    pub effect_type : EffectType,
}

pub fn add_effect(creator : Option<EntityId>, effect_type: EffectType) {
    EFFECT_QUEUE
        .lock()
        .unwrap()
        .push_back(EffectSpawner{
            creator,
            effect_type,
        });
}

pub fn run_effects_queue(gs: &mut State) {
    loop {
        let effect : Option<EffectSpawner> = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(effect) = effect {
            target_applicator(gs, &effect);
        } else {
            break;
        }
    }

    if EFFECT_QUEUE.lock().unwrap().len() > 0 {
        dbg!("ERROR: Finished running effecgs queue but there's still effects left");
    }
}

fn get_effected_entities(gs: &State, targets: &Targets) -> Vec<EntityId> {
    let mut entities: Vec<EntityId> = vec![];
    let map = gs.world.borrow::<UniqueView<Map>>().unwrap();

    match targets {
        Targets::Tile { tile_idx } => {
            for entity in map.tile_content[*tile_idx].iter() {
                entities.push(*entity);
            }
        },
        Targets::Tiles { tiles } => {
            for tile_idx in tiles {
                for entity in map.tile_content[*tile_idx].iter() {
                    entities.push(*entity);
                }  
            }
        },
        Targets::Single { target } => {
            entities.push(*target);
        },
        Targets::Area { target } => {
            entities = target.clone();
        },
    }

    return entities;
}

fn get_effected_tiles(world: &World, targets: &Targets) -> Vec<usize> {
    let mut ret: Vec<usize> = vec![];
    let map = world.borrow::<UniqueView<Map>>().unwrap();

    match targets {
        Targets::Tile { tile_idx } => {
            ret.push(*tile_idx);
            // for entity in map.tile_content[*tile_idx].iter() {
            //     entities.push(*entity);
            // }
        },
        Targets::Tiles { tiles } => {
            ret = tiles.clone();
            // for tile_idx in tiles {
            //     for entity in map.tile_content[*tile_idx].iter() {
            //         entities.push(*entity);
            //     }  
            // }
        },
        Targets::Single { target } => {
            if let Ok(vpos) = world.borrow::<View<Position>>() {
                if let Ok(pos) = vpos.get(*target) {
                    for p in pos.ps.iter() {
                        ret.push(map.point_idx(*p));
                    }
                }
            }
            // entities.push(*target);
        },
        Targets::Area { target } => {
            if let Ok(vpos) = world.borrow::<View<Position>>() {
                for target in target {
                    if let Ok(pos) = vpos.get(*target) {
                        for p in pos.ps.iter() {
                            ret.push(map.point_idx(*p));
                        }
                    }
                }
            }
            // entities = target.clone();
        },
    }

    return ret;
}

fn target_applicator(gs: &mut State, effect : &EffectSpawner) {
    match &effect.effect_type {
        EffectType::Damage { .. } => damage::inflict_damage(gs, effect),
        EffectType::Confusion { .. } => confusion::inflict_confusion(gs, effect),
        EffectType::Fire { .. } => fire::inflict_fire(gs, effect),
        EffectType::PickUp { .. } => inventory::pick_up(gs, effect),
        EffectType::Drop { .. } => inventory::drop_item(gs, effect),
        EffectType::Explore {  } => movement::autoexplore(gs, effect),
        EffectType::Heal { .. } => heal::heal(gs, effect),
        EffectType::Move { .. } => movement::try_move_or_attack(gs, effect, false),
        EffectType::Wait {  } => movement::skip_turn(gs, effect),
        EffectType::Delete { .. } => delete::delete(gs, effect) ,
        EffectType::MoveOrAttack { .. } => movement::try_move_or_attack(gs, effect, true),
    }


    // match &effect.targets {
    //     Targets::Tile{tile_idx} => affect_tile(gs, effect, *tile_idx),
    //     Targets::Tiles{tiles} => tiles.iter().for_each(|tile_idx| affect_tile(gs, effect, *tile_idx)),
    //     Targets::Single{target} => affect_entity(gs, effect, *target),
    //     Targets::Area{target} => target.iter().for_each(|entity| affect_entity(gs, effect, *entity)),
    // }
}

// fn tile_effect_hits_entities(effect: &EffectType) -> bool {
//     match effect {
//         EffectType::Damage{..} => true,
//         EffectType::Confusion{..} => true,
//         EffectType::Fire{..} => true,
//         EffectType::PickUp {  } => false,
//         EffectType::Explore {  } => false,
//         EffectType::Drop {  } => true,
//         EffectType::Heal {..} => true,
//         EffectType::Move {  } => true,
//         EffectType::Wait {  } => true,
//         EffectType::Delete {..} => true,
//     }
// }

// fn affect_tile(gs: &mut State, effect: &EffectSpawner, tile_idx : usize) {
//     if tile_effect_hits_entities(&effect.effect_type) {
//         let mut entities: Vec<EntityId> = vec![];

//         {
//             // let res = &gs.resources;
//             let map = gs.world.borrow::<UniqueView<Map>>().unwrap();

//             for entity in map.tile_content[tile_idx].iter() {
//                 entities.push(*entity);
//             }
//         }

//         for entity in entities{
//             affect_entity(gs, effect, entity);
//         }
//     }

//     // run the effect on tile if applicable
//     match &effect.effect_type {
//         EffectType::Damage{..} => {},
//         EffectType::Confusion{..} => {},
//         EffectType::Fire{..} => fire::inflict_fire_tile(gs, effect, tile_idx),
//         EffectType::PickUp { } => {},
//         EffectType::Explore { } => {},
//         EffectType::Drop { } => {},
//         EffectType::Heal {..} => {}, // todo make this cause a burst of life or something
//         EffectType::Move {  } => movement::try_move(gs, effect, tile_idx),
//         EffectType::Wait {  } => {},
//         EffectType::Delete {..} => {}, 
//     }
// }

// fn affect_entity(gs: &mut State, effect: &EffectSpawner, target: EntityId) {
//     match &effect.effect_type {
//         EffectType::Damage{..} => damage::inflict_damage(gs, effect, target),
//         EffectType::Confusion{..} => confusion::inflict_confusion(gs, effect, target),
//         EffectType::Fire{..} => fire::inflict_fire(gs, effect, target),
//         EffectType::PickUp {  } => inventory::pick_up(gs, effect, target),
//         EffectType::Explore {  } => movement::autoexplore(gs, effect, target),
//         EffectType::Drop {  } => inventory::drop_item(gs, effect, target),
//         EffectType::Heal {..} => heal::heal(gs, effect, target),
//         EffectType::Move {  } => { },
//         EffectType::Wait {  } => movement::skip_turn(gs, effect, target),
//         EffectType::Delete {..} => delete::delete(gs, effect, target),
//     }
// }

/*

Make separate systems for each effect and separate queues
Add

Worflow is: process general queue -> (all the other systems)

 */