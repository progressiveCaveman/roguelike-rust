use rltk::Point;
use shipyard::{UniqueView, View, IntoIter, IntoWithId, ViewMut, AddComponent};
use crate::utils::Turn;
use crate::{entity_factory};
use crate::components::{Position, Spawner, Faction, SpawnerType};

pub fn run_spawner_system(turn: UniqueView<Turn>, vpos: View<Position>, vspawner: View<Spawner>, vfaction: ViewMut<Faction>) {    
    // let mut log = res.get_mut::<GameLog>().unwrap();
    // let turn = res.get::<i32>().unwrap();

    let mut to_spawn: Vec<(Point, i32, SpawnerType)> = vec![];

    for (_, (pos, spawner, faction)) in (&vpos, &vspawner, &vfaction).iter().with_id() { //world.query_mut::<(&Position, &Spawner, &Faction)>() {        
        let fpos = pos.ps.first().unwrap();
        if turn.c % spawner.rate == 0 {
            to_spawn.push((Point { x: fpos.x, y: fpos.y + 1 }, faction.faction, spawner.typ));
        }
    }

    for (p, f, t) in to_spawn.iter() {
        match t {
            SpawnerType::Orc => {
                let e = entity_factory::orc(world, p.x, p.y);
                vfaction.add_component_unchecked(e, Faction {faction: *f});
                // world.add_component(e, Faction {faction: *f}).unwrap();        
            },
            SpawnerType::Fish => {
                entity_factory::fish(world, p.x, p.y);        
            },
        }
    }
}