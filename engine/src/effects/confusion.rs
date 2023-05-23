use super::*;
use crate::components::Confusion;

pub fn inflict_confusion(gs: &mut State, confusion: &EffectSpawner) {
    if let EffectType::Confusion { turns, target } = &confusion.effect_type {
        for entity in get_effected_entities(gs, &target) {
            gs.world.add_component(entity, Confusion{ turns: *turns });
        }
    }
}
