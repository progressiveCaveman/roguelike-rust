use rltk::Point;
use shipyard::{View, IntoIter, IntoWithId, Get, AllStoragesViewMut};
use crate::ai::decisions::{Intent, Task, Target};
use crate::utils::InvalidPoint;
use crate::{entity_factory};
use crate::components::{Position, Tree};

pub fn run_dissasemble_system(mut all_storages: AllStoragesViewMut, vpos: View<Position>, vintent: View<Intent>, vtree: View<Tree>) {
    for (id, (pos, intent)) in (&vpos, &vintent).iter().with_id() {
        if intent.task == Task::Destroy {
            let target = intent.target[0].get_point(vpos);

            if target == Point::invalid_point() {
                continue;
            }
            
            // check distance
            for p in pos.ps {
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(target, p);
                if distance > 1.5 {
                    // dbg!("entity not next to target", distance);
                    continue;
                }
    
                if let Target::ENTITY(e) = intent.target[0] {
                    let mut spawn_log = false;
                    if let Ok(_) = vtree.get(e) {
                        spawn_log = true;
                    }
    
                    let tpoint = if let Ok(p) = vpos.get(e) { 
                        p.ps[0]
                    }else{
                        dbg!("No position");
                        Point::invalid_point()
                    };
    
                    if spawn_log {
                        entity_factory::log(all_storages, tpoint.x, tpoint.y);
                    }
    
                    all_storages.delete_entity(e);
                }
    
                break;
            }
        }
    }
}