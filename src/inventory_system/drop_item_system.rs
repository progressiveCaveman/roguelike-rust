use hecs::*;
use resources::*;
use rltk::Point;
use crate::components::{Equipped, InBackpack, Name, Position, WantsToDropItem, Inventory};
use crate::gamelog::GameLog;

pub fn drop_item(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let player_id = res.get_mut::<Entity>().unwrap();
    let mut to_drop: Vec<(Entity, Entity)> = Vec::new();
    let mut to_remove_wants_drop: Vec<Entity> = Vec::new();

    let mut pos: Point = Point{x: 0, y: 0};
    for (id, wants_drop) in &mut world.query::<&WantsToDropItem>().iter() {
        let p = world.get::<Position>(id).unwrap();
        pos = *p.ps.first().unwrap();
        to_remove_wants_drop.push(id);
        to_drop.push((id, wants_drop.item));

        let item_name = world.get::<Name>(wants_drop.item).unwrap();
        if id == *player_id {
            log.messages.push(format!("You drop the {}", item_name.name));
        }
    }

    for id in to_remove_wants_drop.iter() {
        world.remove_one::<WantsToDropItem>(*id).unwrap();
    }

    for (id, item) in to_drop.iter() {
        if let Ok(mut inv) = world.get_mut::<Inventory>(*id) {
            if let Some(pos) = inv.items.iter().position(|x| *x == *item) {
                inv.items.remove(pos);
            }
        }
        
        let _in_bp = world.remove_one::<InBackpack>(*id);
        let _equipped = world.remove_one::<Equipped>(*id);
        world.insert_one(*id, Position { ps:vec![pos]}).unwrap();
    }
}
