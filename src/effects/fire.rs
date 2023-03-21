use super::*;
use crate::{components::Fire};

pub fn inflict_fire_tile(gs: &mut State, effect: &EffectSpawner, target_idx: usize) {
    let res = &gs.resources;
    let mut map = res.get_mut::<Map>().unwrap();

    if let EffectType::Fire { turns } = effect.effect_type {
        if map.is_flammable(target_idx) {
                map.fire_turns[target_idx] += turns;
        }
    }
}

pub fn inflict_fire(gs: &mut State, effect: &EffectSpawner, target: EntityId) {
    let world = &mut gs.world;

    if let EffectType::Fire { turns } = effect.effect_type {
        let _ = world.add_component(target, Fire{turns});
    }
}
