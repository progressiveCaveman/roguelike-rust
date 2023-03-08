use hecs::*;
use resources::*;
use rltk::Point;
use crate::RunState;
use crate::components::{CombatStats, Player, Name, Inventory, InBackpack, Equipped, Position};
use crate::gamelog::GameLog;

pub fn delete_the_dead(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let mut dead: Vec<Entity> = vec![];
    let mut to_drop_items: Vec<(Entity, Point)> = vec![];

    for (id, (stats, pos, inv)) in &mut world.query::<(&CombatStats, &Position, Option<&Inventory>)>() {
        if stats.hp <= 0 {
            let player = world.get::<Player>(id);
            let name = world.get::<Name>(id);
            match player {
                Err(_) => {
                    dead.push(id);
                    if let Some(inv) = inv {
                        for e in inv.items.iter() {
                            to_drop_items.push((*e, pos.ps[0]));
                        }
                    }
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

    for (e, point) in to_drop_items.iter() {
        let _in_bp = world.remove_one::<InBackpack>(*e);
        let _equipped = world.remove_one::<Equipped>(*e);
        world.insert_one(*e, Position { ps:vec![*point]}).unwrap();    
    }

    for id in dead.iter() {
        let _res = world.despawn(*id);
    }
}
