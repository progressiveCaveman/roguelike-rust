use std::collections::HashMap;

use resources::*;
use rltk::{RandomNumberGenerator, Point, DijkstraMap};
use shipyard::{EntityId, World};
use crate::components::{AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, DealsDamage, EquipmentSlot, Equippable, Item, MeleeDefenseBonus, MeleePowerBonus, Monster, Name, Player, Position, ProvidesHealing, Ranged, Renderable, SerializeMe, Viewshed, Fire, Flammable, Locomotive, PlankHouse, ChiefHouse, FishCleaner, LumberMill, Spawner, Faction, SpatialKnowledge, Inventory, Villager, ItemType, Tree, DijkstraMapToMe, Fish, SpawnerType, LocomotionType};
use crate::gui::Palette;
use crate::{RenderOrder};
use crate::rect::Rect;
use crate::weighted_table::WeightedTable;
use crate::map::{Map, TileType};
use crate::MAPWIDTH;
use crate::systems::system_fire::NEW_FIRE_TURNS;

const MAX_MONSTERS: i32 = 4;

pub fn player(world: &mut World, pos: (i32, i32)) -> EntityId {
    world.add_entity((
        SerializeMe {},
        Position { ps: vec![Point{ x: pos.0, y: pos.1 }]},
        Renderable {
            glyph: rltk::to_cp437('@'),
            fg: Palette::COLOR_PURPLE,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Player,
            ..Default::default()
        },
        Player {},
        Locomotive { mtype: LocomotionType::Ground, speed: 1 },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 20,
            dirty: true
        },
        Name {name: "Blabinou".to_string()},
        CombatStats {max_hp: 30, hp: 30, defense: 2, power: 5, regen_rate: 1},
        SpatialKnowledge {
            tiles: HashMap::new(),
        },
        Inventory {
            capacity: 20,
            items: Vec::new(),
        }
    ))
}

pub fn room_table(depth: i32) -> WeightedTable {
    WeightedTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + depth)
        .add("Confusion Scroll", 2 + depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 2)
        .add("Shield", 2)
        .add("Longsword", depth - 1)
        .add("Tower Shield", depth - 1)
}

pub fn spawn_room(world: &mut World, res: &mut Resources, room: &Rect, depth: i32) {
    let mut possible_targets : Vec<usize> = Vec::new();
    { // Borrow scope - to keep access to the map separated
        let map = res.get::<Map>().unwrap();
        for y in room.y1 + 1 .. room.y2 {
            for x in room.x1 + 1 .. room.x2 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    possible_targets.push(idx);
                }
            }
        }
    }

    spawn_region(world, res, &possible_targets, depth);
}

pub fn spawn_region(ecs: &mut World, _res: &mut Resources, area : &[usize], map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points : HashMap<usize, String> = HashMap::new();
    let mut areas : Vec<usize> = Vec::from(area);

    // Scope to keep the borrow checker happy
    {
        let mut rng = RandomNumberGenerator::new();
        let num_spawns = i32::min(areas.len() as i32, rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3);
        if num_spawns == 0 { return; }

        for _i in 0 .. num_spawns {
            let array_index = if areas.len() == 1 { 0usize } else { (rng.roll_dice(1, areas.len() as i32)-1) as usize };
            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, spawn_table.roll(&mut rng).unwrap());
            areas.remove(array_index);
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        spawn_entity(ecs, &spawn);
    }
}

/// Spawns a named entity (name in tuple.1) at the location in (tuple.0)
fn spawn_entity(ecs: &mut World, spawn : &(&usize, &String)) {
    let x = (*spawn.0 % MAPWIDTH) as i32;
    let y = (*spawn.0 / MAPWIDTH) as i32;

    match spawn.1.as_ref() {
        "Goblin" => goblin(ecs, x, y),
        "Orc" => orc(ecs, x, y),
        "Health Potion" => health_potion(ecs, x, y),
        "Fireball Scroll" => fireball_scroll(ecs, x, y),
        "Confusion Scroll" => confusion_scroll(ecs, x, y),
        "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
        "Dagger" => dagger(ecs, x, y),
        "Shield" => shield(ecs, x, y),
        "Longsword" => longsword(ecs, x, y),
        "Tower Shield" => tower_shield(ecs, x, y),
        _ => unreachable!()
    };
}

/// Monsters

pub fn villager(world: &mut World, x: i32, y:i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('v'),
            fg: Palette::COLOR_RED,
            bg: Palette::MAIN_BG,
            order: RenderOrder::NPC,
            ..Default::default()
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 20,
            dirty: true
        },
        Locomotive { mtype: LocomotionType::Ground, speed: 1 },
        Name {name: "Villager".to_string() },
        BlocksTile {},
        Inventory { capacity: 5, items: Vec::new() },
        SpatialKnowledge { tiles: HashMap::new() },
        Villager {},
    ))
}

pub fn fish(world: &mut World, x: i32, y:i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('f'),
            fg: Palette::COLOR_AMBER,
            bg: Palette::MAIN_BG,
            order: RenderOrder::NPC,
            ..Default::default()
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 2,
            dirty: true
        },
        Locomotive { mtype: LocomotionType::Water, speed: 1 },
        Name {name: "Fish".to_string() },
        Fish {},
        Item { typ: ItemType::Fish }
    ))
}

pub fn orc(world: &mut World, x: i32, y:i32) -> EntityId{
    monster(world, x, y, rltk::to_cp437('o'), "Orc".to_string())
}

pub fn goblin(world: &mut World, x: i32, y:i32) -> EntityId {
    monster(world, x, y, rltk::to_cp437('g'), "Goblin".to_string())
}

pub fn monster(world: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: String) -> EntityId{
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph,
            fg: Palette::COLOR_RED,
            bg: Palette::MAIN_BG,
            order: RenderOrder::NPC,
            ..Default::default()
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true
        },
        Monster {},
        Locomotive { mtype: LocomotionType::Ground, speed: 1 },
        Name {name},
        BlocksTile {},
        CombatStats {max_hp: 8, hp: 8, defense: 1, power: 4, regen_rate: 0},
        Inventory { capacity: 5, items: Vec::new() }
    ))
}

#[allow(dead_code)]
pub fn big_monster(world: &mut World, x: i32, y: i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }, Point{ x: x+1, y }, Point{ x, y: y+1 }, Point{ x: x+1, y: y+1 }]},
        Renderable {
            glyph: rltk::to_cp437('o'),
            fg: Palette::COLOR_RED,
            bg: Palette::MAIN_BG,
            order: RenderOrder::NPC,
            ..Default::default()
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true
        },
        Monster {},
        Locomotive { mtype: LocomotionType::Ground, speed: 1 },
        Name {name: "Monster".to_string()},
        BlocksTile {},
        CombatStats {max_hp: 8, hp: 8, defense: 1, power: 4, regen_rate: 0}
    ))
}

/// consumables

pub fn health_potion(world: &mut World, x: i32, y:i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('p'),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Health potion".to_string()},
        Item {typ: ItemType::Potion},
        ProvidesHealing { heal: 8 },
        Consumable {}
    ))
}

pub fn magic_missile_scroll(world: &mut World, x: i32, y:i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('('),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Magic missile scroll".to_string()},
        Item {typ: ItemType::Scroll},
        Consumable {},
        DealsDamage {damage: 8},
        Ranged {range:6}
    ))
}

pub fn fireball_scroll(world: &mut World, x: i32, y: i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('*'),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Fireball scroll".to_string()},
        Item {typ: ItemType::Scroll},
        Consumable {},
        DealsDamage {damage: 20},
        Ranged {range: 6},
        AreaOfEffect {radius: 3}
    ))
}

pub fn confusion_scroll(world: &mut World, x: i32, y: i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('&'),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Confusion scroll".to_string()},
        Item { typ: ItemType::Scroll },
        Consumable {},
        Ranged {range: 6},
        Confusion {turns: 4}
    ))
}

/// equippables

pub fn dagger(world: &mut World, x: i32, y: i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('│'),
            fg: Palette::COLOR_3,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Dagger".to_string()},
        Item {typ: ItemType::Weapon},
        Equippable {slot: EquipmentSlot::RightHand},
        MeleePowerBonus {power: 4}
    ))
}

pub fn longsword(world: &mut World, x: i32, y: i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('│'),
            fg: Palette::COLOR_3,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Dagger".to_string()},
        Item {typ: ItemType::Shield},
        Equippable {slot: EquipmentSlot::RightHand},
        MeleePowerBonus {power: 8}
    ))
}

pub fn shield(world: &mut World, x: i32, y: i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('°'),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Shield".to_string()},
        Item {typ: ItemType::Shield},
        Equippable {slot: EquipmentSlot::LeftHand},
        MeleeDefenseBonus {defense: 4}
    ))
}

pub fn tower_shield(world: &mut World, x: i32, y: i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('°'),
            fg: Palette::COLOR_4,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Shield".to_string()},
        Item {typ: ItemType::Shield},
        Equippable {slot: EquipmentSlot::LeftHand},
        MeleeDefenseBonus {defense: 8}
    ))
}

pub fn log(world: &mut World, x: i32, y: i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('='),
            fg: Palette::COLOR_CEDAR,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Log".to_string()},
        Item {typ: ItemType::Log},
        Flammable {}
    ))
}

// structures

pub fn spawner(world: &mut World, x: i32, y: i32, faction: i32, typ: SpawnerType, rate: i32) -> EntityId {    
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('&'),
            fg: Palette::FACTION_COLORS[faction as usize],
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Spawner".to_string()},
        Spawner { typ, rate},
        Faction {faction}
    ))
}

pub fn tree(world: &mut World, x: i32, y: i32) -> EntityId {
    world.add_entity((
        Position {ps: vec![Point{ x, y }]},
        Renderable {
            glyph: rltk::to_cp437('|'),
            fg: Palette::COLOR_CEDAR,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Tree".to_string()},
        Flammable {},
        Tree {}
    ))
}

pub fn plank_house(world: &mut World, x: i32, y: i32, width: i32, height: i32) -> EntityId {
    let mut ps = vec![];
    for xi in 0..width {
        for yi in 0..height {
            ps.push(Point{ x: x + xi, y: y+ yi});
        }
    }

    // TODO pick colors for buildings, maybe glyph?

    world.add_entity((
        Position {ps},
        Renderable {
            glyph: rltk::to_cp437('#'),
            fg: Palette::COLOR_CEDAR,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Plank House".to_string()},
        Flammable {},
        PlankHouse { housing_cap: 5, villagers: vec![] },
        BlocksTile {}
    ))
}

pub fn chief_house(world: &mut World, x: i32, y: i32, width: i32, height: i32) -> EntityId {
    let mut ps = vec![];
    for xi in 0..width {
        for yi in 0..height {
            ps.push(Point{ x: x + xi, y: y+ yi});
        }
    }

    // TODO pick colors for buildings, maybe glyph?

    world.add_entity((
        Position {ps},
        Renderable {
            glyph: rltk::to_cp437('#'),
            fg: Palette::COLOR_CEDAR,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "chief_house".to_string()},
        Flammable {},
        ChiefHouse {},
        BlocksTile {}
    ))
}

pub fn fish_cleaner(world: &mut World, x: i32, y: i32, width: i32, height: i32) -> EntityId {
    let mut ps = vec![];
    for xi in 0..width {
        for yi in 0..height {
            ps.push(Point{ x: x + xi, y: y+ yi});
        }
    }

    // TODO pick colors for buildings, maybe glyph?

    world.add_entity((
        Position {ps},
        Renderable {
            glyph: rltk::to_cp437('#'),
            fg: Palette::MAIN_FG,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Fish Cleaner".to_string()},
        Flammable {},
        FishCleaner {},
        BlocksTile {},
        Inventory { capacity: 50, items: Vec::new() },
        DijkstraMapToMe { map: DijkstraMap::new_empty(0, 0, 0.) }
    ))
}

pub fn lumber_mill(world: &mut World, x: i32, y: i32, width: i32, height: i32) -> EntityId {
    let mut ps = vec![];
    for xi in 0..width {
        for yi in 0..height {
            ps.push(Point{ x: x + xi, y: y+ yi});
        }
    }

    // TODO pick colors for buildings, maybe glyph?

    world.add_entity((
        Position {ps},
        Renderable {
            glyph: rltk::to_cp437('#'),
            fg: Palette::COLOR_AMBER,
            bg: Palette::MAIN_BG,
            order: RenderOrder::Items,
            ..Default::default()
        },
        Name {name: "Lumber Mill".to_string()},
        Flammable {},
        LumberMill {},
        BlocksTile {},
        Inventory { capacity: 50, items: Vec::new() },
        DijkstraMapToMe { map: DijkstraMap::new_empty(0, 0, 0.) }
    ))
}

/// misc

pub fn tmp_fireball(world: &mut World) -> EntityId {
    world.add_entity((
        Name {name: "Fireball".to_string()},
        Item {typ: ItemType::Scroll},
        Consumable {},
        DealsDamage {damage: 20},
        Ranged {range: 6},
        AreaOfEffect {radius: 3},
        Fire {turns: NEW_FIRE_TURNS}
    ))
}