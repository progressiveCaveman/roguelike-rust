use hecs::*;
use super::*;
use crate::components::Confusion;

pub fn inflict_confusion(world: &mut World, _res: &mut Resources, confusion: &EffectSpawner, target: Entity) {
    if let EffectType::Confusion { turns } = confusion.effect_type {
        world.insert_one(target, Confusion{turns: turns}).unwrap();
    }
}
