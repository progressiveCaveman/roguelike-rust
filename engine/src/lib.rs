#[macro_use]
extern crate lazy_static;

use components::{
    Equipped, InBackpack, Player, Position, Viewshed, IsCamera,
};
use gamelog::GameLog;
use item_system::{run_drop_item_system, run_item_use_system, run_unequip_item_system};
use map::{Map, TileType};
use rltk::{Point, Rltk};

mod item_system;

pub mod ai;

pub mod gui;

pub mod components;
pub mod entity_factory;
pub mod gamelog;
pub mod map;
pub mod player;
pub mod rect;
pub mod utils;
pub mod weighted_table;

pub mod map_builders;

pub mod systems;
use shipyard::{
    AllStoragesViewMut, EntitiesView, EntityId, Get, Unique, UniqueView, UniqueViewMut, View,
    ViewMut, World,
};
use systems::{
    system_ai_fish, system_ai_monster, system_ai_spawner, system_ai_villager,
    system_dissasemble, system_fire, system_map_indexing, system_melee_combat, system_particle,
    system_pathfinding, system_visibility,
};
use utils::{FrameTime, PPoint, PlayerID, Turn, RNG};

pub mod effects;

pub const SHOW_MAPGEN_ANIMATION: bool = true;
pub const MAPGEN_FRAME_TIME: f32 = 25.0;

pub const TILE_SIZE: usize = 10;
pub const MAPWIDTH: usize = 200;
pub const MAPHEIGHT: usize = 80;
pub const WINDOWWIDTH: usize = 160;
pub const WINDOWHEIGHT: usize = 80;
pub const SCALE: f32 = 1.0;

#[derive(Copy, Clone, PartialEq, Unique)]
pub enum GameMode {
    NotSelected,
    Sim,
    RL,
}



#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum RenderOrder {
    Items,
    NPC,
    Player,
    Particle,
}
pub trait EngineController: 'static {
    fn start(&self, world: &mut World);
    fn update(&self, world: &mut World, ctx: &mut Rltk);
}

pub struct Engine {}
impl Engine {
    pub fn run_systems(world: &mut World, player_turn: bool, ai_turn: bool) {
        if player_turn {
            world.run(system_fire::run_fire_system);
        }
        world.run(system_visibility::run_visibility_system);

        world.run(effects::run_effects_queue);

        if ai_turn {
            world.run(system_pathfinding::run_pathfinding_system);
            world.run(system_ai_spawner::run_spawner_system);
            world.run(system_ai_fish::run_fish_ai);
            world.run(system_ai_villager::run_villager_ai_system);
            world.run(system_ai_monster::run_monster_ai_system);
            // system_ai_monster::run_monster_ai_system(self);
        }

        world.run(effects::run_effects_queue);

        world.run(system_map_indexing::run_map_indexing_system);

        world.run(system_melee_combat::run_melee_combat_system);
        world.run(item_system::run_inventory_system);
        world.run(system_dissasemble::run_dissasemble_system);
        world.run(run_drop_item_system);
        world.run(run_unequip_item_system);
        world.run(run_item_use_system);
        world.run(system_particle::spawn_particles);

        world.run(effects::run_effects_queue);
        world.run(system_map_indexing::run_map_indexing_system);
    }

    pub fn entities_to_delete_on_level_change(world: &mut World) -> Vec<EntityId> {
        let mut ids_to_delete: Vec<EntityId> = Vec::new();

        let entities = world.borrow::<EntitiesView>().unwrap();
        let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;

        let vplayer = world.borrow::<View<Player>>().unwrap();
        let vpack = world.borrow::<View<InBackpack>>().unwrap();
        let vequipped = world.borrow::<View<Equipped>>().unwrap();

        for id in entities.iter() {
            let mut to_delete = true;

            if let Ok(_) = vplayer.get(id) {
                to_delete = false;
            } else if let Ok(backpack) = vpack.get(id) {
                if backpack.owner == player_id {
                    to_delete = false;
                }
            } else if let Ok(equipped) = vequipped.get(id) {
                if equipped.owner == player_id {
                    to_delete = false;
                }
            }

            if to_delete {
                ids_to_delete.push(id);
            }
        }

        ids_to_delete
    }

    pub fn generate_map(world: &mut World, new_depth: i32) {
        // delete all entities
        let ids_to_delete = Self::entities_to_delete_on_level_change(world);
        for id in ids_to_delete {
            world.delete_entity(id);
        }

        // self.mapgen_data.index = 0;
        // self.mapgen_data.timer = 0.0;
        // self.mapgen_data.history.clear();

        // get game mode
        let gamemode = *world.borrow::<UniqueView<GameMode>>().unwrap();

        // Generate map
        let mut map_builder = match gamemode {
            GameMode::NotSelected => {
                map_builders::random_builder(new_depth, (MAPWIDTH as i32, MAPHEIGHT as i32))
            }
            GameMode::Sim => {
                map_builders::village_builder(new_depth, (MAPWIDTH as i32, MAPHEIGHT as i32))
            }
            GameMode::RL => {
                map_builders::rl_builder(new_depth, (MAPWIDTH as i32, MAPHEIGHT as i32))
            }
        };

        map_builder.build_map();

        // self.mapgen_data.history = map_builder.get_map_history();

        let start_pos;
        {
            let mut map = world.borrow::<UniqueViewMut<Map>>().unwrap();
            *map = map_builder.get_map();
            start_pos = map_builder
                .get_starting_position()
                .ps
                .first()
                .unwrap()
                .clone();
        }

        // Spawn monsters and items
        map_builder.spawn_entities(world);

        // Update player position
        world.run(
            |mut ppos: UniqueViewMut<PPoint>,
             player_id: UniqueView<PlayerID>,
             mut vpos: ViewMut<Position>,
             mut vvs: ViewMut<Viewshed>| {
                *ppos = PPoint(Point::new(start_pos.x, start_pos.y));
                if let Ok(pos) = (&mut vpos).get(player_id.0) {
                    pos.ps[0] = ppos.0;
                }

                if let Ok(mut vs) = (&mut vvs).get(player_id.0) {
                    vs.dirty = true;
                }
            },
        );
    }

    pub fn next_level(world: &mut World) {
        // Generate new map
        let current_depth;
        {
            let map = world.borrow::<UniqueViewMut<Map>>().unwrap();
            current_depth = map.depth;
        }
        Self::generate_map(world, current_depth + 1);

        // Notify player
        let mut log = world.borrow::<UniqueViewMut<GameLog>>().unwrap();
        log.messages
            .push("You descend in the staircase".to_string());
    }

    pub fn set_game_mode(world: &mut World, mode: GameMode) {
        world.run(|mut store: AllStoragesViewMut| {
            let player_id: EntityId = store.borrow::<UniqueView<PlayerID>>().unwrap().0;
            let player_is_alive = {
                let entities = store.borrow::<EntitiesView>().unwrap();
                entities.is_alive(player_id)
            };

            dbg!(player_is_alive);

            match mode {
                GameMode::Sim => {
                    if player_is_alive{
                        store.add_component(player_id, IsCamera {});
                    }
                },
                _ => {
                    store.delete_component::<IsCamera>(player_id);
                },
            }

            let mut gamemode = store.borrow::<UniqueViewMut<GameMode>>().unwrap();
            *gamemode = mode;
        });
    }

    pub fn reset_engine(world: &mut World) {
        // Delete everything
        world.clear();

        // Re-add defaults for all uniques
        world.add_unique(Map::new(
            1,
            TileType::Wall,
            (MAPWIDTH as i32, MAPHEIGHT as i32),
        ));
        world.add_unique(PPoint(Point::new(0, 0)));
        world.add_unique(Turn(0));
        world.add_unique(RNG(rltk::RandomNumberGenerator::new()));
    
        let player_id = world
            .run(|mut store: AllStoragesViewMut| entity_factory::player(&mut store, (0, 0)));
        world.add_unique(PlayerID(player_id));
    
        world.add_unique(GameMode::NotSelected);
        world.add_unique(gamelog::GameLog { messages: vec![] });
        world.add_unique(system_particle::ParticleBuilder::new());
        world.add_unique(FrameTime(0.));

        // Generate new map
        Self::generate_map(world, 1);
    } 
}