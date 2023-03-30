use shipyard::{ViewMut, Get, UniqueViewMut, AddComponent};

use super::*;
use crate::{components::{CombatStats}, gamelog::GameLog};

pub fn inflict_damage(gs: &mut State, damage: &EffectSpawner, target: EntityId) {
    let world = &gs.world;
    let mut log = gs.world.borrow::<UniqueViewMut<GameLog>>().unwrap();

    if let EffectType::Damage{amount} = damage.effect_type {
        if let Ok(mut vs) = world.borrow::<ViewMut<CombatStats>>() {
            match (&vs).get(target) {
                Ok(stats) => {
                    let mut stats = stats.clone();
                    stats.hp -= amount;
                    vs.add_component_unchecked(target, stats);
                },
                Err(_e) => {
                    log.messages.push(format!("Damage failed!!"));
                }
            }
        }
    }
}
