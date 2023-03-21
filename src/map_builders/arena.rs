use std::collections::HashMap;

use resources::Resources;
use rltk::{RandomNumberGenerator, Point};

use crate::{SHOW_MAPGEN_ANIMATION, entity_factory, components::SpawnerType};

use super::{MapBuilder, Map, TileType, Position};

pub struct AernaBuilder {
    map : Map,
    starting_position : Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas : HashMap<i32, Vec<usize>>
}

impl MapBuilder for AernaBuilder {
    fn get_map(&mut self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&mut self) -> Position {
        self.starting_position.clone()
    }
    fn build_map(&mut self)  {
        self.build()
    }

    fn spawn_entities(&mut self, world: &mut World, res: &mut Resources) {
        entity_factory::spawner(world, 4, self.map.height / 2, 0, SpawnerType::Orc, 10);
        entity_factory::spawner(world, self.map.width - 5, self.map.height / 2, 1, SpawnerType::Orc, 10);
    }

    fn get_map_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_ANIMATION {
            self.history.push(self.map.clone());
        }
    }
}

impl AernaBuilder {
    pub fn new(new_depth : i32) -> AernaBuilder {
        AernaBuilder{
            map : Map::new(new_depth, TileType::Floor),
            starting_position : Position{ ps: vec![Point{x:0, y:0}] },
            depth : new_depth,
            history: Vec::new(),
            noise_areas : HashMap::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // set edges to be a wall
        for x in 0..self.map.width {
            let idx = self.map.xy_idx(x, 0);
            self.map.tiles[idx] = TileType::Wall;

            let idx = self.map.xy_idx(x, self.map.height - 1);
            self.map.tiles[idx] = TileType::Wall;
        }

        for y in 0..self.map.height {
            let idx = self.map.xy_idx(0, y);
            self.map.tiles[idx] = TileType::Wall;

            let idx = self.map.xy_idx(self.map.width - 1, y);
            self.map.tiles[idx] = TileType::Wall;
        }
        self.take_snapshot();

        // Set the map to grass with a river
        // for y in 1..self.map.height-1 {
        //     for x in 1..self.map.width-1 {
        //         let idx = self.map.xy_idx(x, y);

        //         if y > self.map.height - 10 && y < self.map.height - 3 {
        //             self.map.tiles[idx] = TileType::Water;
        //         } else {
        //             self.map.tiles[idx] = TileType::Grass;
        //         }
        //     }
        // }
    
        // First we completely randomize the map, setting 55% of it to be floor.
        // for y in 1..self.map.height/2 {
        //     for x in 1..self.map.width-1 {
        //         let roll = rng.roll_dice(1, 100);
        //         let idx = self.map.xy_idx(x, y);
        //         if roll > 55 { self.map.tiles[idx] = TileType::Floor } 
        //         // else { self.map.tiles[idx] = TileType::Wall }
        //     }
        // }
        // self.take_snapshot();

        self.starting_position = Position{ ps: vec![Point{x: self.map.width/2, y: self.map.height/2}] };

        return;
    }

}