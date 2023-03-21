use resources::*;
use shipyard::EntityId;

use crate::{components::{Equipped, InBackpack, Name, WantsToUnequipItem, Inventory}, gamelog::GameLog};

pub fn run_unequip_item_system(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let player_id = res.get_mut::<EntityId>().unwrap();
    let mut to_unequip: Vec<(EntityId, EntityId)> = Vec::new();
    let mut to_remove_wants_unequip: Vec<EntityId> = Vec::new();

    for (id, (_, wants_unequip)) in world.query::<(&Inventory, &WantsToUnequipItem)>().iter() {
        to_remove_wants_unequip.push(id);
        to_unequip.push((id, wants_unequip.item));

        if id == *player_id {
            let item_name = world.get::<Name>(wants_unequip.item).unwrap();
            log.messages.push(format!("You unequip the {}", item_name.name));
        }
    }

    for (id, item_id) in to_unequip { //todo will .iter() break this
        world.remove_one::<Equipped>(item_id).unwrap();
        world.insert_one(item_id, InBackpack{owner: id}).unwrap();
    }

    for id in to_remove_wants_unequip {
        world.remove_one::<WantsToUnequipItem>(id).unwrap();
    }
}
