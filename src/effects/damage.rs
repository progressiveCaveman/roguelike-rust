use hecs::*;
use super::*;
use crate::components::{CombatStats};
use crate::gamelog::GameLog;

pub fn inflict_damage(world: &mut World, res: &mut Resources, damage: &EffectSpawner, target: Entity) {
    let mut log = res.get_mut::<GameLog>().unwrap();

    if let EffectType::Damage{amount} = damage.effect_type {

        let stats = world.get_mut::<CombatStats>(target);
        match stats {
            Ok(mut stats) => {
                stats.hp -= amount;
            },
            Err(_e) => {
                log.messages.push(format!("Damage failed!!"));
            }
        }
    }
}
