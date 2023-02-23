use hecs::*;
use resources::*;
use crate::RunState;
use crate::components::{CombatStats, Player, Name};
use crate::gamelog::GameLog;

pub fn delete_the_dead(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let mut dead: Vec<Entity> = vec![];

    for (id, stats) in &mut world.query::<&CombatStats>() {
        if stats.hp <= 0 {
            let player = world.get::<Player>(id);
            let name = world.get::<Name>(id);
            match player {
                Err(_) => {
                    dead.push(id);
                    if let Ok(name) = name {
                        log.messages.push(format!("{} is dead", &name.name));
                    }
                }
                Ok(_p) => {
                    let mut runstate = res.get_mut::<RunState>().unwrap();
                    *runstate = RunState::GameOver;
                }
            }
        }
    }

    for id in dead.iter() {
        let _res = world.despawn(*id);
    }
}
