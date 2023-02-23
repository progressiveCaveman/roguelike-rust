use hecs::*;
use resources::*;
use crate::{components::{CombatStats, Equipped, MeleeDefenseBonus, MeleePowerBonus, Name, Position, WantsToAttack}, systems::particle_system::ParticleBuilder, effects::add_effect, gui::Palette};
use crate::gamelog::GameLog;
use crate::effects::{EffectType, Targets};

pub fn melee_combat(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let mut particle_builder = res.get_mut::<ParticleBuilder>().unwrap();

    let mut to_remove_wants_melee: Vec<Entity> = vec![];

    for (id, (wants_attack, name, stats)) in &mut world.query::<(&WantsToAttack, &Name, &CombatStats)>() {
        if stats.hp > 0 {
            let target_stats = &world.get::<CombatStats>(wants_attack.target).unwrap();
            if target_stats.hp > 0 {
                let mut offensize_bonus = 0;
                for (_item_id, (power_bonus, equipped)) in world.query::<(&MeleePowerBonus, &Equipped)>().iter() {
                    if equipped.owner == id { offensize_bonus += power_bonus.power }
                }

                if target_stats.hp > 0 {
                    let mut defensize_bonus = 0;
                    for (_item_id, (defense_bonus, equipped)) in world.query::<(&MeleeDefenseBonus, &Equipped)>().iter() {
                        if equipped.owner == wants_attack.target { defensize_bonus += defense_bonus.defense }
                    }
                    let damage = i32::max(0, (stats.power + offensize_bonus) - (target_stats.defense + defensize_bonus));
                    
                    let target_name = &world.get::<Name>(wants_attack.target).unwrap();
                    if damage == 0 {
                        log.messages.push(format!("{} is unable to hurt {}", &name.name, &target_name.name));
                    }
                    else {
                        log.messages.push(format!("{} hits {} for {} hp", &name.name, &target_name.name, damage));
                        add_effect(
                            Some(id), 
                            EffectType::Damage{ amount: damage },
                            Targets::Single{ target: wants_attack.target }
                        );
                    }

                    let pos = &world.get::<Position>(wants_attack.target);
                    if let Ok(pos) = pos {
                        for pos in pos.ps.iter() {
                            particle_builder.request(pos.x, pos.y, 0.0, 0.0, Palette::COLOR_4, Palette::MAIN_BG, rltk::to_cp437('‼'), 250.0);
                        }
                    }
                }
            }
        }
        to_remove_wants_melee.push(id);
    }

    for id in to_remove_wants_melee.iter() {
        let _res = world.remove_one::<WantsToAttack>(*id);
    }
}
