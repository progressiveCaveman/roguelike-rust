mod drop_item_system;
pub use drop_item_system::system_drop_item;

mod item_use_system;
pub use item_use_system::item_use;

mod unequip_item_system;
pub use unequip_item_system::unequip_item;

use hecs::*;
use resources::*;
use crate::ai::decisions::{Intent, Task, Target};
use crate::components::{WantsToPickupItem, Position, InBackpack, Name, Inventory};
use crate::gamelog::{GameLog};

pub fn inventory(world: &mut World, res: &mut Resources) {
    let mut need_in_backpack: Vec<(Entity, WantsToPickupItem)> = Vec::new();
    let mut need_pickup: Vec<(Entity, Intent)> = Vec::new();
    let mut to_deposit: Vec<(Entity, Inventory, Intent)> = Vec::new();

    for (id, (_, wants_pickup)) in &mut world.query::<(&Inventory, &WantsToPickupItem)>() {
        need_in_backpack.push((id, *wants_pickup));
    }

    for (id, (inv, intent)) in &mut world.query::<(&Inventory, &Intent)>() {
        if intent.task == Task::PickUpItem {
            need_pickup.push((id, intent.clone()));
        }
        if intent.task == Task::DepositItemToInventory {
            to_deposit.push((id, inv.clone(), intent.clone()));
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

    for (id, inv, intent) in to_deposit.iter() {
        if let Target::ENTITY(item) = intent.target[0] {
            if let Target::ENTITY(target) = intent.target[1] {
                drop_item_system::drop_item(world, id, &item);
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