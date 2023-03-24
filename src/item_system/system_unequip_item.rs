use shipyard::{UniqueView, View, ViewMut, IntoIter, IntoWithId, Get, Remove, AddComponent};

use crate::{components::{Equipped, InBackpack, Name, WantsToUnequipItem, Inventory, Player}, gamelog::GameLog};

pub fn run_unequip_item_system(log: UniqueView<GameLog>, vplayer: View<Player>, vinv: View<Inventory>, vwants: ViewMut<WantsToUnequipItem>, vequip: ViewMut<Equipped>, vbackpack: ViewMut<InBackpack>, vname: View<Name>) {
    for (id, (_, wants_unequip)) in (&vinv, &vwants).iter().with_id() {
        vwants.remove(id);
        vequip.remove(wants_unequip.item);
        vbackpack.add_component_unchecked(wants_unequip.item, InBackpack { owner: id });

        if let Ok(_) = vplayer.get(id){
            if let Ok(item_name) = vname.get(wants_unequip.item) {
                log.messages.push(format!("You unequip the {}", item_name.name));
            }
        }
    }
}
