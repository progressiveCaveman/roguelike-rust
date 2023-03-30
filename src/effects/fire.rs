use shipyard::{UniqueViewMut};

use super::*;
use crate::{components::Fire, map::Map};

pub fn inflict_fire_tile(gs: &mut State, effect: &EffectSpawner, tile_idx: usize) {
    let mut map = gs.world.borrow::<UniqueViewMut<Map>>().unwrap();

    if let EffectType::Fire { turns } = effect.effect_type {
        if map.is_flammable(tile_idx) {
            map.fire_turns[tile_idx] += turns;
        }
    }
}

pub fn inflict_fire(gs: &mut State, effect: &EffectSpawner, target: EntityId) {
    let world = &mut gs.world;

    if let EffectType::Fire { turns } = effect.effect_type {
        let _ = world.add_component(target, Fire{turns});
    }
}
