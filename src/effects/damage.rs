use shipyard::{ViewMut, Get};

use super::*;
use crate::components::{CombatStats};

pub fn inflict_damage(gs: &mut State, damage: &EffectSpawner, target: EntityId) {
    let world = &gs.world;
    let mut log = gs.get_log();

    if let EffectType::Damage{amount} = damage.effect_type {
        if let Ok(s) = world.borrow::<ViewMut<CombatStats>>() {
            match (&s).get(target) {
                Ok(mut stats) => {
                    stats.hp -= amount;
                },
                Err(_e) => {
                    log.messages.push(format!("Damage failed!!"));
                }
            }
        }
    }
}
