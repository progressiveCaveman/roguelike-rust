mod system_drop_item;
pub use system_drop_item::system_drop_item;

mod system_item_use;
pub use system_item_use::item_use;

mod system_unequip_item;
pub use system_unequip_item::unequip_item;

use hecs::*;
use resources::*;
use crate::ai::decisions::{Intent, Task, Target};
use crate::components::{WantsToPickupItem, Position, InBackpack, Name, Inventory};
use crate::gamelog::{GameLog};

pub fn inventory(world: &mut World, res: &mut Resources) {
    let mut need_in_backpack: Vec<(Entity, WantsToPickupItem)> = Vec::new();
    let mut need_pickup: Vec<(Entity, Intent)> = Vec::new();
    let mut to_deposit: Vec<(Entity, Intent)> = Vec::new();

    for (id, (_, wants_pickup)) in &mut world.query::<(&Inventory, &WantsToPickupItem)>() {
        need_in_backpack.push((id, *wants_pickup));
    }

    for (id, (_, intent)) in &mut world.query::<(&Inventory, &Intent)>() {
        if intent.task == Task::PickUpItem {
            need_pickup.push((id, intent.clone()));
        }
        if intent.task == Task::DepositItemToInventory {
            to_deposit.push((id, intent.clone()));
        }
    }

    for (id, wants_pickup) in need_in_backpack.iter() {
        pick_up(world, res, id, wants_pickup.item);
    }

    for (id, intent) in need_pickup.iter() {
        if let Target::ENTITY(e) = intent.target[0] {
            pick_up(world, res, id, e);
        }
    }

    for (id, intent) in to_deposit.iter() {
        if let Target::ENTITY(item) = intent.target[0] {
            if let Target::ENTITY(target) = intent.target[1] {
                system_drop_item::drop_item(world, id, &item);
                pick_up(world, res, &target, item);
            }   
        }
    }
}

fn pick_up(world: &mut World, res: &mut Resources, id: &Entity, item: Entity) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let player_id = res.get::<Entity>().unwrap();

    if let Ok(mut inv) = world.get_mut::<Inventory>(*id) {
        inv.items.push(item);
    } else {
        dbg!("Entity has no inventory");
    }

    let _res = world.remove_one::<Position>(item);
    let _r = world.insert_one(item, InBackpack {owner: *id});

    if *id == *player_id {
        let name = world.get::<Name>(item).unwrap();
        log.messages.push(format!("You pick up the {}", name.name));
    }

    let _re = world.remove_one::<WantsToPickupItem>(*id);
}