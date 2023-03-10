use hecs::*;
use rltk::Point;
use crate::ai::decisions::{Intent, Task, Target};
use crate::utils::InvalidPoint;
use crate::{entity_factory, State};
use crate::components::{Position, Tree};

pub fn run_dissasemble_system(gs: &mut State) {
    let world = &mut gs.world;

    let mut wants_to_destroy: Vec<(Entity, Vec<Point>, Intent)> = vec![];

    for (id, (pos, intent)) in &mut world.query::<(&Position, &Intent)>() {
        if intent.task == Task::Destroy {
            wants_to_destroy.push((id, pos.ps.to_vec(), intent.clone()));
        }
    }

    let mut wants_despawn: Vec<Entity> = vec![];

    for (_, pos, intent) in wants_to_destroy.iter() {
        let target = intent.target[0].get_point(world);

        if target == Point::invalid_point() {
            continue;
        }
        
        // check distance
        for p in pos {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(target, *p);
            if distance > 1.5 {
                // dbg!("entity not next to target", distance);
                continue;
            }

            if let Target::ENTITY(e) = intent.target[0] {
                let mut spawn_log = false;
                if let Ok(_) = world.get::<Tree>(e) { 
                    spawn_log = true;
                }

                let tpoint = if let Ok(p) = world.get::<Position>(e) { 
                    p.ps[0]
                }else{
                    dbg!("No position");
                    Point::invalid_point()
                };

                if spawn_log {
                    entity_factory::log(world, tpoint.x, tpoint.y);
                }

                wants_despawn.push(e);
            }

            break;
        }
    }

    for e in wants_despawn {
        let _res = world.despawn(e);
    }
}