use hecs::*;
use resources::*;

use crate::{components::{Equipped, InBackpack, Name, WantsToUnequipItem}, gamelog::GameLog};

pub fn unequip_item(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let player_id = res.get_mut::<Entity>().unwrap();
    let mut to_unequip: Vec<(Entity, Entity)> = Vec::new();
    let mut to_remove_wants_unequip: Vec<Entity> = Vec::new();

    for (id, wants_unequip) in world.query::<&WantsToUnequipItem>().iter() {
        to_remove_wants_unequip.push(id);
        to_unequip.push((id, wants_unequip.item));

        if id == *player_id {
            let item_name = world.get::<Name>(wants_unequip.item).unwrap();
            log.messages.push(format!("You unequip the {}", item_name.name));
        }
    }

    for (id, item_id) in to_unequip {
        world.remove_one::<Equipped>(item_id).unwrap();
        world.insert_one(item_id, InBackpack{owner: id}).unwrap();
    }

    for id in to_remove_wants_unequip {
        world.remove_one::<WantsToUnequipItem>(id).unwrap();
    }
}
