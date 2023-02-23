use hecs::*;
use super::*;
use crate::{components::Fire};

pub fn inflict_fire_tile(_world: &mut World, res: &mut Resources, effect: &EffectSpawner, target_idx: usize) {
    let mut map = res.get_mut::<Map>().unwrap();

    if let EffectType::Fire { turns } = effect.effect_type {
        if map.is_flammable(target_idx) {
                map.fire_turns[target_idx] += turns;
        }
    }
}

pub fn inflict_fire(world: &mut World, _res: &mut Resources, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Fire { turns } = effect.effect_type {
        let _ = world.insert_one(target, Fire{turns});
    }
}
