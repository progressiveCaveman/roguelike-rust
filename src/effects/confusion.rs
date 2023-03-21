use super::*;
use crate::components::Confusion;

pub fn inflict_confusion(gs: &mut State, confusion: &EffectSpawner, target: EntityId) {
    let world = &mut gs.world;
    if let EffectType::Confusion { turns } = confusion.effect_type {
        world.add_component(target, Confusion{turns: turns});
    }
}
