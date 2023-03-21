use shipyard::{View, ViewMut, IntoIter, IntoWithId, Get, UniqueView, UniqueViewMut, AllStoragesViewMut, Remove};
use crate::RunState;
use crate::components::{CombatStats, Player, Name, Inventory, InBackpack, Equipped, Position};
use crate::gamelog::GameLog;

pub fn run_cleanup_system(log: UniqueView<GameLog>, runstate: UniqueViewMut<RunState>, mut all_storages: AllStoragesViewMut, vpos: View<Position>, vstats: ViewMut<CombatStats>, vinv: View<Inventory>, vplayer: View<Player>, vname: View<Name>, vpack: ViewMut<InBackpack>, vequip: ViewMut<Equipped>) {
    for (id, (pos, stats)) in (&vpos, &vstats).iter().with_id() {
        if stats.hp <= 0 {
            let player = vplayer.get(id);// world.get::<Player>(id);
            let name = vname.get(id); // world.get::<Name>(id);
            match player {
                Err(_) => { // not a player
                    if let Ok(inv) = vinv.get(id) {
                        for e in inv.items.iter() {
                            (vpack, vequip).remove(*e);
                            all_storages.add_component(*e, Position { ps: vec![pos.ps[0]] });
                            // to_drop_items.push((*e, pos.ps[0]));
                        }
                    }

                    all_storages.delete_entity(id);
                    
                    if let Ok(name) = name {
                        log.messages.push(format!("{} is dead", &name.name));
                    }
                }
                Ok(_p) => {
                    *runstate = RunState::GameOver;
                }
            }
        }
    }
}
