use hecs::*;
use resources::*;
use crate::effects::add_effect;
use crate::gui::Palette;
use crate::{components::Position, gamelog::GameLog, systems::system_particle::ParticleBuilder};
use crate::components::{WantsToUseItem, CombatStats, ProvidesHealing, Name, Consumable, DealsDamage, AreaOfEffect, Confusion, Equippable, Equipped, InBackpack, Fire, Inventory};
use crate::map::Map;
use crate::effects::{EffectType, Targets};

pub fn item_use(world: &mut World, res: &mut Resources) {
    let mut log = res.get_mut::<GameLog>().unwrap();
    let player_id = res.get::<Entity>().unwrap();
    let map = res.get::<Map>().unwrap();
    let mut p_builder = res.get_mut::<ParticleBuilder>().unwrap();
    let mut to_remove: Vec<(Entity, Entity)> = Vec::new();
    let mut to_remove_wants_use: Vec<Entity> = Vec::new();
    let mut to_heal: Vec<(Entity, ProvidesHealing)> = Vec::new();
    let mut to_unequip: Vec<(Entity, Name, Entity)> = Vec::new();
    let mut to_equip: Vec<(Entity, Equippable, Name, Entity)> = Vec::new();

    for (id, use_item) in &mut world.query::<&WantsToUseItem>().iter() {
        let mut used_item = true;

        // Find all targets
        let mut targets: Vec<Entity> = Vec::new();
        let mut target_tiles: Vec<usize> = Vec::new();
        match use_item.target {
            None => targets.push(*player_id),
            Some(t) => {
                match world.get::<AreaOfEffect>(use_item.item) {
                    Err(_e) => {
                        // Single target
                        let idx = map.xy_idx(t.x, t.y);
                        for entity in map.tile_content[idx].iter() {
                            let stats = world.get::<CombatStats>(*entity);
                            match stats {
                                Err(_e) => {}
                                Ok(_stats) => { targets.push(*entity) }
                            }
                        }
                    }
                    Ok(aoe) => {
                        // AOE
                        let mut affected_tiles = rltk::field_of_view(t, aoe.radius, &*map);
                        affected_tiles.retain(|p| p.x > 0 && p.x < map.width-1 && p.y > 0 && p.y < map.height-1);
                        for pt in affected_tiles.iter() {
                            let idx = map.xy_idx(pt.x, pt.y);
                            target_tiles.push(idx);
                            for entity in map.tile_content[idx].iter() {
                                let stats = world.get::<CombatStats>(*entity);
                                match stats {
                                    Err(_e) => {}
                                    Ok(_stats) => { targets.push(*entity) }
                                }
                            }
                            p_builder.request(pt.x, pt.y, 0.0, 0.0, Palette::COLOR_3, Palette::MAIN_BG, rltk::to_cp437('o'), 250.0)
                        }
                    }
                }
            }
        }

        // Apply fire if it applies fire
        let item_fires = world.get::<Fire>(use_item.item);
        match item_fires {
            Err(_e) => {}
            Ok(fire) => {
                add_effect(
                    Some(id),
                    EffectType::Fire { turns: fire.turns },
                    Targets::Tiles { tiles: target_tiles }
                );                            
                used_item = true;
            }
        }

        // Apply heal if it provides healing
        let item_heals = world.get::<ProvidesHealing>(use_item.item);
        match item_heals {
            Err(_e) => {}
            Ok(healer) => {
                used_item = false;
                for target in targets.iter() {
                    let stats = world.get_mut::<CombatStats>(*target);
                    match stats {
                        Err(_e) => {},
                        Ok(_stats) => {
                            to_heal.push((*target, *healer));
                            if id == *player_id {
                                let name = world.get::<Name>(use_item.item).unwrap();
                                log.messages.push(format!("You use the {}, healing {} hp", name.name, healer.heal));
                            }
                            used_item = true;

                            if let Ok(pos) = world.get::<Position>(*target) {
                                for pos in pos.ps.iter() {
                                    p_builder.request(pos.x, pos.y, 0.0, -3.0, Palette::COLOR_3, Palette::MAIN_BG, rltk::to_cp437('♥'), 1000.0)
                                }
                            }
                        }
                    }
                }
            }
        }
        to_remove_wants_use.push(id);

        // Apply damage to target if it deals damage
        let deals_damage = world.get::<DealsDamage>(use_item.item);
        match deals_damage {
            Err(_e) => {}
            Ok(dd) => {
                used_item = false;
                for target in targets.iter() {
                    add_effect(
                        Some(id),
                        EffectType::Damage{ amount: dd.damage },
                        Targets::Single{ target: *target }
                    );
                    if id == *player_id {
                        let monster_name = world.get::<Name>(*target).unwrap();
                        let item_name = world.get::<Name>(use_item.item).unwrap();
                        log.messages.push(format!("You use {} on {}, dealing {} hp", item_name.name, monster_name.name, dd.damage));
                    }
                    used_item = true;

                    if let Ok(pos) = world.get::<Position>(*target) {
                        for pos in pos.ps.iter() {
                            p_builder.request(pos.x, pos.y, 0.0, 0.0, Palette::COLOR_4, Palette::MAIN_BG, rltk::to_cp437('‼'), 250.0)
                        }
                    }
                }
            }
        }

        // Apply confusion
        let confusion = world.get::<Confusion>(use_item.item);
        match confusion {
            Err(_e) => {},
            Ok(confusion) => {
                used_item = false;
                for target in targets.iter() {
                    add_effect(
                        Some(id), 
                        EffectType::Confusion { turns: confusion.turns }, 
                        Targets::Single { target: *target }
                    );
                    if id == *player_id {
                        let monster_name = world.get::<Name>(*target).unwrap();
                        let item_name = world.get::<Name>(use_item.item).unwrap();
                        log.messages.push(format!("You use {} on {}, confusing them", item_name.name, monster_name.name));
                    }
                    used_item = true;

                    if let Ok(pos) = world.get::<Position>(*target) {
                        for pos in pos.ps.iter() {
                            p_builder.request(pos.x, pos.y, 0.0, 0.0, Palette::COLOR_3, Palette::MAIN_BG, rltk::to_cp437('?'), 300.0)
                        }
                    }
                }
            }
        }

        // Remove item if it's consumable
        let consumable = world.get::<Consumable>(use_item.item);
        match consumable {
            Err(_e) => {}
            Ok(_) => {
                if used_item {
                    to_remove.push((id, use_item.item));
                }
            }
        }

        // Equip if item is equippable
        let equippable = world.get::<Equippable>(use_item.item);
        match equippable {
            Err(_e) => {}
            Ok(equippable) => {
                let target = targets[0];
                
                // Unequip already equipped item
                for (id, (equipped, name)) in world.query::<(&Equipped, &Name)>().iter() {
                    if equipped.owner == target && equipped.slot == equippable.slot {
                        to_unequip.push((id, name.clone(), target));
                    }
                }

                // Actually equip item
                let item_name = (*world.get::<Name>(use_item.item).unwrap()).clone();
                to_equip.push((use_item.item, *equippable, item_name, target));
            }
        }
    }

    for (id, item) in to_remove {
        if let Ok(mut inv) = world.get_mut::<Inventory>(id) {
            if let Some(pos) = inv.items.iter().position(|x| *x == item) {
                inv.items.remove(pos);
            }
        }

        world.despawn(item).unwrap();
    }

    for id in to_remove_wants_use {
        world.remove_one::<WantsToUseItem>(id).unwrap();
    }

    for (id, heals) in to_heal {
        let mut stats = world.get_mut::<CombatStats>(id).unwrap();
        stats.hp = i32::min(stats.hp + heals.heal, stats.max_hp);
    }

    for (id, name, target) in to_unequip {
        world.remove_one::<Equipped>(id).unwrap();
        world.insert_one(id, InBackpack{owner: target}).unwrap();
        if target == *player_id {
            log.messages.push(format!("You unequip your {}", name.name));
        }
    }

    for (id, equippable, name, target) in to_equip {
        world.insert_one(id, Equipped{owner: target, slot: equippable.slot}).unwrap();
        world.remove_one::<InBackpack>(id).unwrap();
        if target == *player_id {
            log.messages.push(format!("You equip your {}", name.name));
        }
    }
}
