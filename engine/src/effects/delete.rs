use super::*;

pub fn delete(gs: &mut State, effect: &EffectSpawner) {
    if let EffectType::Delete { entity } = effect.effect_type {
        gs.world.delete_entity(entity);
    }
}
