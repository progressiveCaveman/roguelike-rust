use shipyard::{ViewMut, Get};

use super::*;
use crate::components::{CombatStats};
use crate::gamelog::GameLog;

pub fn inflict_damage(gs: &mut State, damage: &EffectSpawner, target: EntityId) {
    let world = &gs.world;
    let res = &gs.resources;
    let mut log = res.get_mut::<GameLog>().unwrap();

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
