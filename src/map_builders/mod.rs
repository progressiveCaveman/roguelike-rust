mod arena;
use self::arena::AernaBuilder;

mod simple_map;
use self::simple_map::SimpleMapBuilder;

mod bsp_dungeon;
use self::bsp_dungeon::BspDungeonBuilder;

mod bsp_interior;
use self::bsp_interior::BspInteriorBuilder;

mod bsp_farm;
use self::bsp_farm::BspFarmBuilder;

mod cellular_automata;
use self::cellular_automata::CellularAutomataBuilder;

mod drunkardsbombingrun;
use self::drunkardsbombingrun::DrunkardsBombingRunBuilder;

mod village;
use self::village::VillageBuilder;

mod common;
use common::*;
use shipyard::World;

use crate::rect::Rect;
use crate::map::{Map, TileType};
use crate::components::Position;

pub struct MapGenData {
    pub history: Vec<Map>,
    pub index: usize,
    pub timer: f32
}

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, world: &mut World);
    fn get_map(&mut self) -> Map;
    fn get_starting_position(&mut self) -> Position;
    fn get_map_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 5);
    match builder {
        1 => Box::new(BspDungeonBuilder::new(new_depth)),
        2 => Box::new(BspInteriorBuilder::new(new_depth)),
        3 => Box::new(CellularAutomataBuilder::new(new_depth)),
        4 => Box::new(DrunkardsBombingRunBuilder::new(new_depth)),
        5 => Box::new(BspFarmBuilder::new(new_depth)),
        _ => Box::new(SimpleMapBuilder::new(new_depth))
    }
}

pub fn village_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    Box::new(VillageBuilder::new(new_depth))
}

pub fn rl_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    Box::new(DrunkardsBombingRunBuilder::new(new_depth))
}

pub fn arena_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    Box::new(AernaBuilder::new(new_depth))
}