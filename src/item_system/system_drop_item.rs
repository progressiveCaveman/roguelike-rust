use shipyard::{EntityId, View, IntoIter, IntoWithId, Get, ViewMut, Remove, UniqueViewMut};
use crate::components::{Name, WantsToDropItem, Player};
use crate::effects::{add_effect, EffectType, Targets};
use crate::gamelog::GameLog;

pub fn run_drop_item_system(mut log: UniqueViewMut<GameLog>, vplayer: View<Player>, mut vwants: ViewMut<WantsToDropItem>, vname: View<Name>) {
    let mut to_drop: Vec<(EntityId, EntityId)> = Vec::new();

    for (id, wants_drop) in vwants.iter().with_id() {
        to_drop.push((id, wants_drop.item));

        if let Ok(_) = vplayer.get(id) {
            if let Ok(item_name) = vname.get(wants_drop.item){
                log.messages.push(format!("You drop the {}", item_name.name));
            }
        }
    }

    for (id, item) in to_drop.iter() {
        vwants.remove(*id);
        add_effect(
            Some(*id), 
            EffectType::Drop {}, 
            Targets::Single { target: *item }
        );
    }
}