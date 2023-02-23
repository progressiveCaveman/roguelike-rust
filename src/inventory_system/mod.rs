mod drop_item_system;
pub use drop_item_system::drop_item;

mod item_use_system;
pub use item_use_system::item_use;

mod unequip_item_system;
pub use unequip_item_system::unequip_item;

use hecs::*;
use resources::*;
use crate::components::{WantsToPickupItem, Position, InBackpack, Name};
use crate::gamelog::{GameLog};

pub fn inventory(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let player_id = res.get::<Entity>().unwrap();
    let mut need_in_backpack: Vec<(Entity, WantsToPickupItem)> = Vec::new();

    for (id, wants_pickup) in &mut world.query::<&WantsToPickupItem>() {
        need_in_backpack.push((id, *wants_pickup));
    }

    for (id, wants_pickup) in need_in_backpack.iter() {
        let _res = world.remove_one::<Position>(wants_pickup.item);
        let _r = world.insert_one(wants_pickup.item, InBackpack {owner: wants_pickup.collected_by});

        if wants_pickup.collected_by == *player_id {
            let name = world.get::<Name>(wants_pickup.item).unwrap();
            log.messages.push(format!("You pick up the {}", name.name));
        }

        let _re = world.remove_one::<WantsToPickupItem>(*id);
    }
}
