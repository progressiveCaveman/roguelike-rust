mod system_drop_item;
use shipyard::{View, IntoIter, IntoWithId, ViewMut, Remove};
pub use system_drop_item::run_drop_item_system;

mod system_item_use;
pub use system_item_use::run_item_use_system;

mod system_unequip_item;
pub use system_unequip_item::run_unequip_item_system;

use crate::ai::decisions::{Intent, Task, Target};
use crate::components::{WantsToPickupItem, Inventory};
use crate::effects::{add_effect, EffectType, Targets};

pub fn run_inventory_system(vinv: View<Inventory>, vwants: View<WantsToPickupItem>, mut vintent: ViewMut<Intent>) {
    let to_remove_intent = vec![];

    for (id, (_, wants_pickup)) in (&vinv, &vwants).iter().with_id() {
        add_effect(
            Some(id), 
            EffectType::PickUp {}, 
            Targets::Single { target: wants_pickup.item }
        );
    }

    for (id, (_, intent)) in (&vinv, &vintent).iter().with_id() {
        if intent.task == Task::PickUpItem {
            if let Target::ENTITY(e) = intent.target[0] {
                add_effect(
                    Some(id), 
                    EffectType::PickUp {}, 
                    Targets::Single { target: e }
                );
            }
        }
        if intent.task == Task::DepositItemToInventory {
            if let Target::ENTITY(item) = intent.target[0] {
                if let Target::ENTITY(target) = intent.target[1] {
                    add_effect(
                        Some(id), 
                        EffectType::Drop {}, 
                        Targets::Single { target: item }
                    );
                    add_effect(
                        Some(target), 
                        EffectType::PickUp {}, 
                        Targets::Single { target: item }
                    );
                }   
            }
        }
    }

    for id in to_remove_intent {
        vintent.remove(id);
    }
}