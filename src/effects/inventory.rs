use shipyard::{ViewMut, Get, UniqueView, UniqueViewMut};

use super::*;
use crate::{components::{Position, Inventory, InBackpack, WantsToPickupItem, Name, Equipped}, utils::{WorldGet, PlayerID}, gamelog::GameLog};

pub fn pick_up(gs: &mut State, effect: &EffectSpawner, target: EntityId) {
    if let Some(id) = effect.creator {
        let mut log = gs.world.borrow::<UniqueViewMut<GameLog>>().unwrap();
        let player_id = gs.world.borrow::<UniqueView<PlayerID>>().unwrap().0;
    
        let shouldReturn = gs.world.run(|mut positions: ViewMut<Position>| {
            if let Err(_) = (&mut positions).get(target) {
                dbg!("Entity doesn't have a position");
                return true;
            }
            false
        });

        if shouldReturn { return; }

        if let Err(_) = gs.world.get::<Position>(id) {
            dbg!("Entity doesn't have a position");
            return;
        }
    
        if let Ok(name) = gs.world.get::<Name>(id) {
            if let Ok(inv) = gs.world.get::<Inventory>(id) {
                inv.items.push(target);

                let mut entities: Vec<EntityId> = vec![];
                for e1 in inv.items.iter() {
                    let mut dup = false;
                    for e2 in entities.iter() {
                        if e2 == e1 {
                            dup = true;
                            // println!("ERROR: Duplicate item in {} inventory", name.name);
                            return;
                        }
                    }
                    if !dup {
                        entities.push(*e1);
                    }
                }
            } else {
                dbg!("Entity has no inventory");
            }
        }
    
        let _res = gs.world.remove::<Position>(target);
        let _r = gs.world.add_component(target, InBackpack {owner: id});
    
        if id == player_id {
            let name = gs.world.get::<Name>(target).unwrap();
            log.messages.push(format!("You pick up the {}", name.name));
        }
    
        let _re = gs.world.remove::<WantsToPickupItem>(id);    
    }
}

pub fn drop_item(gs: &mut State, effect: &EffectSpawner, target: EntityId) {
    if let Some(id) = effect.creator {
        let world = &mut gs.world;

        let pos = if let Ok(p) = world.get::<Position>(id) {
            p.any_point()
        }else{
            unreachable!()
        };
        if let Ok(mut inv) = world.get::<Inventory>(id) {
            if let Some(pos) = inv.items.iter().position(|x| *x == target) {
                inv.items.remove(pos);
            }
        }
        
        let _in_bp = world.remove::<InBackpack>(id);
        let _equipped = world.remove::<Equipped>(id);
        world.add_component(target, Position { ps:vec![pos]});
    }
}