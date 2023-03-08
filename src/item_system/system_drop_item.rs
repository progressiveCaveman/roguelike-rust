use hecs::*;
use resources::*;
use crate::components::{Name, WantsToDropItem};
use crate::gamelog::GameLog;

use super::drop_item;

pub fn run_drop_item_system(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let player_id = res.get_mut::<Entity>().unwrap();
    let mut to_drop: Vec<(Entity, Entity)> = Vec::new();

    for (id, wants_drop) in &mut world.query::<&WantsToDropItem>().iter() {
        to_drop.push((id, wants_drop.item));

        let item_name = world.get::<Name>(wants_drop.item).unwrap();
        if id == *player_id {
            log.messages.push(format!("You drop the {}", item_name.name));
        }
    }

    for (id, item) in to_drop.iter() {
        world.remove_one::<WantsToDropItem>(*id).unwrap();
        drop_item(world, id, item);
    }
}