use hecs::*;
use super::*;
use crate::{components::{Fire, Position, Inventory, InBackpack, WantsToPickupItem, Name}, gamelog::GameLog};

// pub fn pick_up(_world: &mut World, res: &mut Resources, effect: &EffectSpawner, target_idx: usize) {
//     let mut map = res.get_mut::<Map>().unwrap();

//     if let EffectType::Fire { turns } = effect.effect_type {
//         if map.is_flammable(target_idx) {
//                 map.fire_turns[target_idx] += turns;
//         }
//     }
// }


pub fn pick_up(world: &mut World, res: &mut Resources, effect: &EffectSpawner, target: Entity) {
    if let Some(id) = effect.creator {
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
