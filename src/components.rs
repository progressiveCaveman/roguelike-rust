use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use hecs::*;
use rltk::{self, Point, DijkstraMap};

use crate::{RenderOrder, map::{TileType, Map}, utils::InvalidPoint};

/// Basic UI components

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub ps: Vec<Point>
}

impl Position {
    pub fn any_point(&self) -> Point {
        if self.ps.len() > 0 {
            *self.ps.first().unwrap()
        } else {
            Point::invalid_point()
        }
    }

    pub fn idxes(&self, map: &Map) -> Vec<usize> {
        self.ps.iter().map(|it| map.point_idx(*it)).collect()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: rltk::RGBA,
    pub bg: rltk::RGBA,
    pub render: bool,
    pub always_render: bool,
    pub order: RenderOrder
}

impl Default for Renderable {
    fn default() -> Self {
        Renderable {
            glyph: rltk::to_cp437(' '),
            fg: rltk::RGBA{r: 1., g: 1., b: 1., a: 1.},
            bg: rltk::RGBA{r: 0., g: 0., b: 0., a: 1.},
            render: true,
            always_render: false,
            order: RenderOrder::Player
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool
}

impl Viewshed {
    pub fn is_visible(&self, idx: Point) -> bool {
        for p in self.visible_tiles.iter() {
            if p.x == idx.x && p.y == idx.y {
                return true
            }
        }

        false
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Name {
    pub name: String
}

/// Entity properties

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Player {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Monster {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Villager {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Faction {
    pub faction: i32
}

/// Structures

// #[derive(Copy, Clone, Debug, PartialEq)]
pub struct PlankHouse {
    pub housing_cap: i32,
    pub villagers: Vec<Entity>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ChiefHouse {
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct LumberMill {
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FishCleaner {
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Spawner {
    pub rate: i32
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Tree {}

/// Labors?

/// Entity properties

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Locomotive {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BlocksTile {}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
    pub regen_rate: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Inventory {
    pub capacity: i32,
    pub items: Vec<Entity>
}

impl Inventory {
    pub fn count_type(&self, world: &World, item_type: ItemType) -> i32 {
        let mut count = 0;
        for e in self.items.iter() {
            if let Ok(item) = world.get::<Item>(*e) {
                if item.typ == item_type {
                    count += 1;
                }
            }
        }

        return count;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SpatialKnowledge {
    pub tiles: HashMap<usize, (TileType, Vec<Entity>)>,
}

pub struct DijkstraMapToMe {
    pub map: DijkstraMap
}

/// Entity intents

#[derive(Clone, Copy)]
pub struct WantsToAttack {
    pub target: Entity
}

#[derive(Clone, Copy)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity
}

#[derive(Clone, Copy)]
pub struct WantsToDropItem {
    pub item: Entity
}

pub struct WantsToUnequipItem {
    pub item: Entity
}

pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<rltk::Point>
}

/// Inventory components

#[derive(PartialEq, Copy, Clone)]
pub enum EquipmentSlot { RightHand, LeftHand }

#[derive(Copy, Clone)]
pub struct Equippable {
    pub slot: EquipmentSlot
}

pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot
}

pub struct InBackpack {
    pub owner: Entity
}

/// Item properties

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ItemType {
    Log,
    Shield,
    Weapon,
    Potion,
    Scroll
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Item {
    pub typ: ItemType
}

pub struct Consumable {}

pub struct MeleePowerBonus {
    pub power: i32
}

pub struct MeleeDefenseBonus {
    pub defense: i32
}

#[derive(Clone, Copy)]
pub struct ProvidesHealing {
    pub heal: i32
}

pub struct Ranged {
    pub range: i32
}

#[derive(Clone, Copy)]
pub struct DealsDamage {
    pub damage: i32
}

#[derive(Clone, Copy)]
pub struct Confusion {
    pub turns: i32
}

pub struct AreaOfEffect {
    pub radius: i32
}

/// Fire components

#[derive(Clone, Copy)]
pub struct Fire {
    pub turns: i32
}

#[derive(Clone, Copy)]
pub struct Flammable {}

/// Save components

pub struct SerializeMe {}

pub struct Lifetime {
    pub ms: f32
}

/// Particle components

pub struct Velocity {
    pub x: f32,
    pub y: f32
}

pub struct Particle {
    pub float_x: f32,
    pub float_y: f32
}
