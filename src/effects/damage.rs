use shipyard::{ViewMut, Get, UniqueViewMut, AddComponent};

use super::*;
use crate::{components::{CombatStats}, gamelog::GameLog};

pub fn inflict_damage(gs: &mut State, damage: &EffectSpawner) {
    let world = &gs.world;
    let mut log = gs.world.borrow::<UniqueViewMut<GameLog>>().unwrap();

    if let EffectType::Damage{amount, target } = &damage.effect_type {
        if let Ok(mut vs) = world.borrow::<ViewMut<CombatStats>>() {
            for target in get_effected_entities(gs, &target) {
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
}
