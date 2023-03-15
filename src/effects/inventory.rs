use hecs::*;
use super::*;
use crate::{components::{Position, Inventory, InBackpack, WantsToPickupItem, Name, Equipped}, gamelog::GameLog};

pub fn pick_up(gs: &mut State, effect: &EffectSpawner, target: Entity) {
    if let Some(id) = effect.creator {
        let world = &mut gs.world;
        let res = &gs.resources;

        let mut log = res.get_mut::<GameLog>().unwrap();
        let player_id = res.get::<Entity>().unwrap();
    
        if let Ok(_) = world.get_mut::<Position>(id) {
    
        } else {
            dbg!("Entity doesn't have a position");
            return;
        }
    
        if let Ok(mut inv) = world.get_mut::<Inventory>(id) {
            inv.items.push(target);
        } else {
            dbg!("Entity has no inventory");
        }
    
        let _res = world.remove_one::<Position>(target);
        let _r = world.insert_one(target, InBackpack {owner: id});
    
        if id == *player_id {
            let name = world.get::<Name>(target).unwrap();
            log.messages.push(format!("You pick up the {}", name.name));
        }
    
        let _re = world.remove_one::<WantsToPickupItem>(id);    
    }
}

pub fn drop_item(gs: &mut State, effect: &EffectSpawner, target: Entity) {
    if let Some(id) = effect.creator {
        let world = &mut gs.world;
        let res = &gs.resources;

        let pos = if let Ok(p) = world.get::<Position>(id) {
            p.any_point()
        }else{
            unreachable!()
        };

        if let Ok(mut inv) = world.get_mut::<Inventory>(id) {
            if let Some(pos) = inv.items.iter().position(|x| *x == target) {
                inv.items.remove(pos);
            }
        }
        
        let _in_bp = world.remove_one::<InBackpack>(id);
        let _equipped = world.remove_one::<Equipped>(id);
        world.insert_one(target, Position { ps:vec![pos]}).unwrap();
    }
}