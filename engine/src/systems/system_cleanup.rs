use crate::components::{CombatStats, Equipped, InBackpack, Inventory, Name, Player, Position};
use crate::effects::{add_effect, EffectType};
use crate::gamelog::GameLog;
use shipyard::{
    AddComponent, EntityId, Get, IntoIter, IntoWithId, Remove, UniqueViewMut, View, ViewMut,
};

pub fn run_cleanup_system(
    mut log: UniqueViewMut<GameLog>,
    mut vpos: ViewMut<Position>,
    vstats: ViewMut<CombatStats>,
    vinv: View<Inventory>,
    vplayer: View<Player>,
    vname: View<Name>,
    mut vpack: ViewMut<InBackpack>,
    mut vequip: ViewMut<Equipped>,
) {
    let mut to_add_pos: Vec<(EntityId, Position)> = vec![];

    for (id, (pos, stats)) in (&vpos, &vstats).iter().with_id() {
        if stats.hp <= 0 {
            let player = vplayer.get(id);
            let name = vname.get(id);
            match player {
                Err(_) => {
                    // not a player
                    if let Ok(inv) = vinv.get(id) {
                        for e in inv.items.iter() {
                            vpack.remove(*e);
                            vequip.remove(*e);
                            to_add_pos.push((
                                *e,
                                Position {
                                    ps: vec![pos.ps[0]],
                                },
                            ));
                        }
                    }

                    add_effect(None, EffectType::Delete { entity: id });

                    if let Ok(name) = name {
                        log.messages.push(format!("{} is dead", &name.name));
                    }
                }
                Ok(_p) => {
                    todo!("Game over");
                    // *runstate = RunState::GameOver;
                }
            }
        }
    }

    for (e, p) in to_add_pos.iter() {
        vpos.add_component_unchecked(*e, p.clone());
    }
}
