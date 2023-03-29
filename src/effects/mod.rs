use std::sync::Mutex;
use std::collections::VecDeque;

mod damage;
pub use damage::inflict_damage;

mod confusion;
pub use confusion::inflict_confusion;

mod fire;
pub use fire::inflict_fire;

mod heal;

mod inventory;
pub use inventory::pick_up;

mod movement;

use shipyard::{EntityId, Component};

use crate::State;

lazy_static! {
    pub static ref EFFECT_QUEUE : Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

pub enum EffectType { 
    Damage { amount : i32 },
    Confusion { turns: i32 },
    Fire { turns: i32 },
    PickUp { },
    Drop { },
    Explore { },
    Heal { amount: i32 },
    Move {},
}

#[derive(Clone)]
pub enum Targets {
    Tile { tile_idx: usize},
    Tiles { tiles: Vec<usize> },
    Single { target: EntityId },
    Area { target: Vec<EntityId> },
}

pub struct EffectSpawner {
    pub creator : Option<EntityId>,
    pub effect_type : EffectType,
    pub targets : Targets
}

pub fn add_effect(creator : Option<EntityId>, effect_type: EffectType, targets : Targets) {
    EFFECT_QUEUE
        .lock()
        .unwrap()
        .push_back(EffectSpawner{
            creator,
            effect_type,
            targets
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
}

fn target_applicator(gs: &mut State, effect : &EffectSpawner) {
    match &effect.targets {
        Targets::Tile{tile_idx} => affect_tile(gs, effect, *tile_idx),
        Targets::Tiles{tiles} => tiles.iter().for_each(|tile_idx| affect_tile(gs, effect, *tile_idx)),
        Targets::Single{target} => affect_entity(gs, effect, *target),
        Targets::Area{target} => target.iter().for_each(|entity| affect_entity(gs, effect, *entity)),
    }
}

fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage{..} => true,
        EffectType::Confusion{..} => true,
        EffectType::Fire{..} => true,
        EffectType::PickUp {  } => false,
        EffectType::Explore {  } => false,
        EffectType::Drop {  } => false,
        EffectType::Heal {..} => true,
        EffectType::Move {  } => false,
    }
}

fn affect_tile(gs: &mut State, effect: &EffectSpawner, tile_idx : usize) {
    if tile_effect_hits_entities(&effect.effect_type) {
        let mut entities: Vec<EntityId> = vec![];

        {
            // let res = &gs.resources;
            let map = gs.get_map();//res.get::<Map>().unwrap();

            for entity in map.tile_content[tile_idx].iter() {
                entities.push(*entity);
            }
        }

        for entity in entities{
            affect_entity(gs, effect, entity);
        }
    }

    // run the effect on tile if applicable
    match &effect.effect_type {
        EffectType::Damage{..} => {},
        EffectType::Confusion{..} => {},
        EffectType::Fire{..} => fire::inflict_fire_tile(gs, effect, tile_idx),
        EffectType::PickUp { } => {},
        EffectType::Explore { } => {},
        EffectType::Drop { } => {},
        EffectType::Heal {..} => {}, // todo make this cause a burst of life or something
        EffectType::Move {  } => movement::try_move(gs, effect, tile_idx), 
    }
}

fn affect_entity(gs: &mut State, effect: &EffectSpawner, target: EntityId) {
    match &effect.effect_type {
        EffectType::Damage{..} => damage::inflict_damage(gs, effect, target),
        EffectType::Confusion{..} => confusion::inflict_confusion(gs, effect, target),
        EffectType::Fire{..} => fire::inflict_fire(gs, effect, target),
        EffectType::PickUp {  } => inventory::pick_up(gs, effect, target),
        EffectType::Explore {  } => movement::autoexplore(gs, effect, target),
        EffectType::Drop {  } => inventory::drop_item(gs, effect, target),
        EffectType::Heal {..} => heal::heal(gs, effect, target),
        EffectType::Move {  } => {},
    }
}

/*

Make separate systems for each effect and separate queues
Add

Worflow is: process general queue -> (all the other systems)

 */