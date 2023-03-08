use hecs::*;
use resources::Resources;
use rltk::Point;
use crate::{entity_factory, RunState};
use crate::components::{Position, Spawner, Faction};

pub fn run_spawner_system(world: &mut World, res: &mut Resources) {
    let runstate: &RunState = &res.get::<RunState>().unwrap();
    if *runstate != RunState::AiTurn { return; }
    
    // let mut log = res.get_mut::<GameLog>().unwrap();
    let turn = res.get::<i32>().unwrap();

    let mut to_spawn: Vec<(Point, i32)> = vec![];

    for (_, (pos, spawner, faction)) in world.query_mut::<(&Position, &Spawner, &Faction)>() {        
        let fpos = pos.ps.first().unwrap();
        if *turn % spawner.rate == 0 {
            to_spawn.push((Point { x: fpos.x, y: fpos.y + 1 }, faction.faction));
        }
    }

    for (p, f) in to_spawn.iter() {
        let e = entity_factory::orc(world, p.x, p.y);
        world.insert_one(e, Faction {faction: *f}).unwrap();
    }
}