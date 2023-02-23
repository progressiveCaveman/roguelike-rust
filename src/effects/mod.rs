use std::sync::Mutex;
use std::collections::VecDeque;
use hecs::*;
use resources::*;

mod damage;
pub use damage::inflict_damage;

mod confusion;
pub use confusion::inflict_confusion;

mod fire;
pub use fire::inflict_fire;

use crate::Map;

lazy_static! {
    pub static ref EFFECT_QUEUE : Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

pub enum EffectType { 
    Damage { amount : i32 },
    Confusion { turns: i32 },
    Fire { turns: i32 }
}

#[derive(Clone)]
pub enum Targets {
    Tile { tile_idx: usize},
    Tiles { tiles: Vec<usize> },
    Single { target: Entity },
    Area { target: Vec<Entity> }
}

pub struct EffectSpawner {
    pub creator : Option<Entity>,
    pub effect_type : EffectType,
    pub targets : Targets
}

pub fn add_effect(creator : Option<Entity>, effect_type: EffectType, targets : Targets) {
    EFFECT_QUEUE
        .lock()
        .unwrap()
        .push_back(EffectSpawner{
            creator,
            effect_type,
            targets
        });
}

pub fn run_effects_queue(ecs : &mut World, res: &mut Resources) {
    loop {
        let effect : Option<EffectSpawner> = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(effect) = effect {
            target_applicator(ecs, res, &effect);
        } else {
            break;
        }
    }
}

fn target_applicator(ecs : &mut World, res: &mut Resources, effect : &EffectSpawner) {
    match &effect.targets {
        Targets::Tile{tile_idx} => affect_tile(ecs, res, effect, *tile_idx),
        Targets::Tiles{tiles} => tiles.iter().for_each(|tile_idx| affect_tile(ecs, res, effect, *tile_idx)),
        Targets::Single{target} => affect_entity(ecs, res, effect, *target),
        Targets::Area{target} => target.iter().for_each(|entity| affect_entity(ecs, res, effect, *entity)),
    }
}

fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    match effect {
        EffectType::Damage{..} => true,
        EffectType::Confusion{..} => true,
        EffectType::Fire{..} => true,
    }
}

fn affect_tile(ecs: &mut World, res: &mut Resources, effect: &EffectSpawner, tile_idx : usize) {
    if tile_effect_hits_entities(&effect.effect_type) {
        let mut entities: Vec<Entity> = vec![];

        {
            let map = res.get::<Map>().unwrap();

            for entity in map.tile_content[tile_idx].iter() {
                entities.push(*entity);
            }
        }

        for entity in entities{
            affect_entity(ecs, res, effect, entity);
        }
    }

    // run the effect on tile if applicable
    match &effect.effect_type {
        EffectType::Damage{..} => {},
        EffectType::Confusion{..} => {},
        EffectType::Fire{..} => fire::inflict_fire_tile(ecs, res, effect, tile_idx),
    }
}

fn affect_entity(ecs: &mut World, res: &mut Resources, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage{..} => damage::inflict_damage(ecs, res, effect, target),
        EffectType::Confusion{..} => confusion::inflict_confusion(ecs, res, effect, target),
        EffectType::Fire{..} => fire::inflict_fire(ecs, res, effect, target),
    }
}