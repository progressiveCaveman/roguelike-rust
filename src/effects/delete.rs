use super::*;

pub fn delete(gs: &mut State, _: &EffectSpawner, target: EntityId) {
    gs.world.delete_entity(target);
}
