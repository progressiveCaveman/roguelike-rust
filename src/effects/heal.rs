use hecs::*;
use super::*;
use crate::components::{CombatStats};

pub fn heal(gs: &mut State, effect: &EffectSpawner, target: Entity) {
    let world = &gs.world;

    if let Ok(mut stats) = world.get_mut::<CombatStats>(target) {
        if let EffectType::Heal{amount} = effect.effect_type {
            stats.hp = i32::min(stats.hp + amount, stats.max_hp);
        }
    }
}
