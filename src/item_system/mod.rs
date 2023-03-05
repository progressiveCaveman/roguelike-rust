mod drop_item_system;
pub use drop_item_system::drop_item;

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

    for (id, (_, wants_pickup)) in &mut world.query::<(&Inventory, &WantsToPickupItem)>() {
        need_in_backpack.push((id, *wants_pickup));
    }

    for (id, (_, intent)) in &mut world.query::<(&Inventory, &Intent)>() {
        if intent.task == Task::PickUpItem {
            need_pickup.push((id, intent.clone()));
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
}

fn pick_up(world: &mut World, res: &mut Resources, id: &Entity, item: Entity) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let player_id = res.get::<Entity>().unwrap();

    if let Ok(mut inv) = world.get_mut::<Inventory>(*id) {
        inv.items.push(item);
    }

    let _res = world.remove_one::<Position>(item);
    let _r = world.insert_one(item, InBackpack {owner: *id});

    if *id == *player_id {
        let name = world.get::<Name>(item).unwrap();
        log.messages.push(format!("You pick up the {}", name.name));
    }

    let _re = world.remove_one::<WantsToPickupItem>(*id);
}