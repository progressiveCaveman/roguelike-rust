use rltk::Point;
use shipyard::{UniqueView, View, IntoIter, IntoWithId, AllStoragesViewMut};
use crate::utils::Turn;
use crate::{entity_factory};
use crate::components::{Position, Spawner, Faction, SpawnerType};

pub fn run_spawner_system(mut store: AllStoragesViewMut) {    
    let mut to_spawn: Vec<(Point, i32, SpawnerType)> = vec![];

    {
        let turn = store.borrow::<UniqueView<Turn>>().unwrap();

        let vpos = store.borrow::<View<Position>>().unwrap();
        let vspawner = store.borrow::<View<Spawner>>().unwrap();
        let vfaction = store.borrow::<View<Faction>>().unwrap();

        for (_, (pos, spawner, faction)) in (&vpos, &vspawner, &vfaction).iter().with_id() {     
            let fpos = pos.ps.first().unwrap();
            if turn.0 % spawner.rate == 0 {
                to_spawn.push((Point { x: fpos.x, y: fpos.y + 1 }, faction.faction, spawner.typ));
            }
        }
    }

    for (p, f, t) in to_spawn.iter() {
        match t {
            SpawnerType::Orc => {
                let e = entity_factory::orc(&mut store, p.x, p.y);
                store.add_component(e, Faction {faction: *f});
            },
            SpawnerType::Fish => {
                entity_factory::fish(&mut store, p.x, p.y);        
            },
        }
    }
}