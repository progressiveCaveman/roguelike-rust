use crate::components::{Faction, Position, Spawner, SpawnerType, Actor};
use crate::entity_factory;
use crate::utils::Turn;
use rltk::Point;
use shipyard::{AllStoragesViewMut, IntoIter, IntoWithId, UniqueView, View, ViewMut, Get};

pub fn run_spawner_system(mut store: AllStoragesViewMut) {
    let mut to_spawn: Vec<(Point, Faction, SpawnerType)> = vec![];

    {
        let turn = store.borrow::<UniqueView<Turn>>().unwrap();

        let vpos = store.borrow::<View<Position>>().unwrap();
        let vspawner = store.borrow::<View<Spawner>>().unwrap();
        let vactor = store.borrow::<View<Actor>>().unwrap();

        for (_, (pos, spawner, actor)) in (&vpos, &vspawner, &vactor).iter().with_id() {
            let fpos = pos.ps.first().unwrap();
            if turn.0 % spawner.rate == 0 {
                to_spawn.push((
                    Point {
                        x: fpos.x,
                        y: fpos.y + 1,
                    },
                    actor.faction,
                    spawner.typ,
                ));
            }
        }
    }

    for (p, f, t) in to_spawn.iter() {
        match t {
            SpawnerType::Orc => {
                let e = entity_factory::orc(&mut store, p.x, p.y);
                store.run(|mut vactor: ViewMut<Actor>|{
                    if let Ok(mut actor) = (&mut vactor).get(e){
                        actor.faction = *f;
                    } else {
                        dbg!("Error: Orc isn't an actor, this shouldn't happen");
                    }
                });
            }
            SpawnerType::Fish => {
                entity_factory::fish(&mut store, p.x, p.y);
            }
        }
    }
}
