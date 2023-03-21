use shipyard::{ViewMut, Get};

use super::*;
use crate::components::{CombatStats};

pub fn heal(gs: &mut State, effect: &EffectSpawner, target: EntityId) {
    if let EffectType::Heal{amount} = effect.effect_type {
        gs.world.run(|mut stats: ViewMut<CombatStats>| {
            if let Ok(mut stats) = (&mut stats).get(target) {
                stats.hp = i32::min(stats.hp + amount, stats.max_hp);
            }
        });
    }
}