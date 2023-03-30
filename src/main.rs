#[macro_use]
extern crate lazy_static;

use map::TileType;
use rltk::{RGBA};
use rltk::{Rltk, GameState, RltkBuilder, Point};

mod item_system;

pub mod ai;

pub mod gui;
use gui::gui_menus;
use gui::camera;

mod player;
mod map;
mod components;
mod rect;
mod gamelog;
mod input_handler;
mod entity_factory;
mod weighted_table;
mod utils;

pub mod map_builders;
use map_builders::MapGenData;

pub mod systems;
use shipyard::{EntityId, World, ViewMut, Get, Unique, UniqueView, UniqueViewMut, AllStoragesViewMut, EntitiesViewMut, View};
use systems::{system_cleanup, system_ai_villager, system_dissasemble, system_fire, system_map_indexing, system_melee_combat, system_ai_monster, system_particle, system_visibility, system_ai_spawner, system_pathfinding, system_ai_fish};

pub mod effects;

use components::{Position, WantsToUseItem, WantsToDropItem, Ranged, Viewshed, WantsToUnequipItem};
use map::Map;
use gamelog::GameLog;
use item_system::{run_drop_item_system, run_item_use_system, run_unequip_item_system};
use utils::{PlayerID, Turn};

use crate::components::{Player, InBackpack, Equipped};
use crate::utils::{PPoint, RNG, FrameTime};

const SHOW_MAPGEN_ANIMATION: bool = true;
const MAPGEN_FRAME_TIME: f32 = 25.0;

const TILE_SIZE: usize = 10;
const MAPWIDTH: usize = 150;
const MAPHEIGHT: usize = 80;
const WINDOWWIDTH: usize = 160;
const WINDOWHEIGHT: usize = 80;
const SCALE: f32 = 1.0;

#[derive(Copy, Clone, PartialEq, Unique)]
pub enum GameMode{
    NotSelected,
    Sim,
    RL,
}

#[derive(Copy, Clone, PartialEq, Unique)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    AiTurn,
    ShowInventory,
    ShowItemActions {item: EntityId},
    ShowTargeting {range: i32, item: EntityId},
    MainMenu {menu_selection: gui_menus::MainMenuSelection},
    SaveGame,
    NextLevel,
    GameOver,
    MapGenAnimation
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum RenderOrder {
    Items,
    NPC,
    Player,
    Particle
}

pub struct State {
    world: World,
    // resources: Resources,
    mapgen_data: MapGenData, 
    autorun: bool,
    wait_frames: i32
}

impl State {
    fn run_systems(&mut self) {
        let runstate: RunState = *self.world.borrow::<UniqueViewMut<RunState>>().unwrap();

        if runstate == RunState::PlayerTurn {
            self.world.run(system_fire::run_fire_system);
        }
        self.world.run(system_visibility::run_visibility_system);

        if runstate == RunState::AiTurn { 
            self.world.run(system_pathfinding::run_pathfinding_system);
            self.world.run(system_ai_spawner::run_spawner_system);
            self.world.run(system_ai_fish::run_fish_ai);
            system_ai_villager::run_villager_ai_system(self);
            self.world.run(system_ai_monster::run_monster_ai_system);
            // system_ai_monster::run_monster_ai_system(self);   
        }
        self.world.run(system_map_indexing::run_map_indexing_system);

        self.world.run(system_melee_combat::run_melee_combat_system);
        self.world.run(item_system::run_inventory_system);
        self.world.run(system_dissasemble::run_dissasemble_system);
        self.world.run(run_drop_item_system);
        self.world.run(run_unequip_item_system);
        self.world.run(run_item_use_system);
        self.world.run(system_map_indexing::run_map_indexing_system);

        effects::run_effects_queue(self);
        self.world.run(system_particle::spawn_particles);
    }

    fn entities_to_delete_on_level_change(&mut self) -> Vec<EntityId> {
        let mut ids_to_delete: Vec<EntityId> = Vec::new();

        let entities = self.world.borrow::<EntitiesViewMut>().unwrap();
        let player_id = self.world.borrow::<UniqueView<PlayerID>>().unwrap().0;

        let vplayer = self.world.borrow::<View<Player>>().unwrap();
        let vpack = self.world.borrow::<View<InBackpack>>().unwrap();
        let vequipped = self.world.borrow::<View<Equipped>>().unwrap();

        for id in entities.iter() {
            let mut to_delete = true;

            if let Ok(_) = vplayer.get(id) {
                to_delete = false;
            }else if let Ok(backpack) = vpack.get(id) {
                if backpack.owner == player_id { to_delete = false; }
            }else if let Ok(equipped) = vequipped.get(id) {
                if equipped.owner == player_id { to_delete = false; }
            }

            if to_delete { ids_to_delete.push(id); }
        }

        ids_to_delete
    }

    fn generate_map(&mut self, new_depth: i32) {

        // delete all entities
        let ids_to_delete = self.entities_to_delete_on_level_change();
        for id in ids_to_delete {
            self.world.delete_entity(id);
        }

        self.mapgen_data.index = 0;
        self.mapgen_data.timer = 0.0;
        self.mapgen_data.history.clear();

        // get game mode
        let gamemode = *self.world.borrow::<UniqueView<GameMode>>().unwrap();//self.resources.get::<GameMode>().unwrap();

        // Generate map
        let mut map_builder = match gamemode {
            GameMode::NotSelected => map_builders::random_builder(new_depth),
            GameMode::Sim => map_builders::village_builder(new_depth),
            GameMode::RL => map_builders::rl_builder(new_depth),
        };

        map_builder.build_map();

        self.mapgen_data.history = map_builder.get_map_history();

        let start_pos;
        {
            let mut map = self.world.borrow::<UniqueViewMut<Map>>().unwrap();
            *map = map_builder.get_map();
            start_pos = map_builder.get_starting_position().ps.first().unwrap().clone();
        }

        // Spawn monsters and items
        map_builder.spawn_entities(&mut self.world);

        // Update player position
        let mut player_position = self.world.borrow::<UniqueViewMut<PPoint>>().unwrap();
        *player_position = PPoint(Point::new(start_pos.x, start_pos.y));
        let player_id = self.world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
        let mut vpos = self.world.borrow::<ViewMut<Position>>().unwrap();
        let player_pos_comp = (&mut vpos).get(player_id).unwrap();
        player_pos_comp.ps[0].x = start_pos.x;
        player_pos_comp.ps[0].y = start_pos.y;

        // update uniqueview ppos
        let mut ppos = self.world.borrow::<UniqueViewMut<PPoint>>().unwrap();
        ppos.0 = player_pos_comp.ps[0];

        // Mark viewshed as dirty
        // let player_vs = self.world.get_mut::<Viewshed>(*player_id);
        self.world.run(|mut vs: ViewMut<Viewshed>| {
            if let Ok(mut vs) = (&mut vs).get(player_id) {
                vs.dirty = true; 
            }
        });
    }

    fn next_level(&mut self) {
        // Generate new map
        let current_depth;
        {
            let map = self.world.borrow::<UniqueViewMut<Map>>().unwrap();
            current_depth = map.depth;
        }
        self.generate_map(current_depth + 1);

        // Notify player
        let mut log = self.world.borrow::<UniqueViewMut<GameLog>>().unwrap();
        log.messages.push("You descend in the staircase".to_string());
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        self.world.clear();

        // Create player
        let player_id = self.world.run(|mut store: AllStoragesViewMut|{entity_factory::player(&mut store, (0, 0))});//entity_factory::player(&mut self.world, (0, 0));
        self.world.add_unique(PPoint(Point::new(0, 0)));
        self.world.add_unique(PlayerID(player_id));

        // Generate new map
        self.generate_map(1);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.set_active_console(1);
        // ctx.cls();
        let (x, y) = ctx.get_char_size();
        for ix in 0..x{
            for iy in 0..y{
                ctx.set(ix, iy, RGBA::from_u8(0, 0, 0, 0), RGBA::from_u8(0, 0, 0, 0), 32)
            }
        }
        ctx.set_active_console(0);
        ctx.cls();

        {
            let mut i = self.world.borrow::<UniqueViewMut<FrameTime>>().unwrap();
            i.0 = ctx.frame_time_ms;
        }

        self.world.run(system_particle::update_particles);
        // system_particle::update_particles(&mut self.world, &mut self.resources, ctx);

        let mut new_runstate: RunState = *self.world.borrow::<UniqueViewMut<RunState>>().unwrap();

        match new_runstate {
            RunState::MainMenu{..} => {}
            RunState::GameOver => {}
            _ => {
                camera::render_camera(self, ctx);
                gui::draw_gui(self, ctx);
            }
        }

        match new_runstate {
            RunState::PreRun => {
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_runstate = input_handler::handle_input(self, ctx);

                if new_runstate == RunState::AwaitingInput && self.autorun {
                    self.wait_frames += 1;
                    if self.wait_frames >= 10 {
                        self.wait_frames = 0;
                        new_runstate = RunState::PlayerTurn
                    }
                }
            }
            RunState::PlayerTurn => {
                self.run_systems();
                new_runstate = RunState::AiTurn;
            }
            RunState::AiTurn => {
                {
                    let mut turn = self.world.borrow::<UniqueViewMut<Turn>>().unwrap();
                    turn.0 += 1;

                    // let map = &mut self.resources.get_mut::<Map>().unwrap();
                    // let gamemode = *self.resources.get::<GameMode>().unwrap();
                    // if gamemode == GameMode::Sim{
                    //     map.refresh_influence_maps(self, *turn);
                    // }
                }
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui_menus::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Cancel => { new_runstate = RunState::AwaitingInput }
                    gui::ItemMenuResult::Selected => {
                        new_runstate = RunState::ShowItemActions{ item: result.1.unwrap() }
                    }
                }
            }
            RunState::ShowItemActions{item} => {
                let result = gui_menus::show_item_actions(&mut self.world, item, ctx);
                match result {
                    gui_menus::ItemActionSelection::NoSelection => {}
                    gui_menus::ItemActionSelection::Used => {
                        let mut to_add_wants_use_item: Vec<EntityId> = Vec::new();
                        {
                            let player_id = self.world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
                            let vranged = self.world.borrow::<ViewMut<Ranged>>().unwrap();
                            match vranged.get(item) {
                                Ok(is_item_ranged) => {
                                    new_runstate = RunState::ShowTargeting{range:is_item_ranged.range, item};
                                }
                                Err(_) => {
                                    to_add_wants_use_item.push(player_id);
                                    new_runstate = RunState::PlayerTurn;
                                }
                            }
                        }

                        for id in to_add_wants_use_item.iter() {
                            self.world.add_component(*id, WantsToUseItem {item, target: None});
                        }
                    }
                    gui_menus::ItemActionSelection::Dropped => {
                        let player_id = self.world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
                        self.world.add_component(player_id, WantsToDropItem {item});
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui_menus::ItemActionSelection::Unequipped => {
                        let player_id = self.world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
                        self.world.add_component(player_id, WantsToUnequipItem{item});
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui_menus::ItemActionSelection::Cancel => { new_runstate = RunState::ShowInventory}
                }
            }
            RunState::ShowTargeting{range, item} => {
                let res = gui::ranged_target(self, ctx, range);
                match res.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let player_id = self.world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
                        self.world.add_component(player_id, WantsToUseItem{item, target: res.1});
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MainMenu{..} => {
                let result = gui_menus::main_menu(self, ctx);
                match result {
                    gui_menus::MainMenuResult::NoSelection{selected} => {new_runstate = RunState::MainMenu{menu_selection: selected}}
                    gui_menus::MainMenuResult::Selection{selected} => {
                        match selected {
                            gui_menus::MainMenuSelection::Roguelike => {
                                {
                                    let mut gamemode = self.world.borrow::<UniqueViewMut<GameMode>>().unwrap();
                                    *gamemode = GameMode::RL;
                                }
                                self.generate_map(1);

                                new_runstate = RunState::MapGenAnimation
                            }
                            gui_menus::MainMenuSelection::Simulator => {
                                {
                                    let mut gamemode = self.world.borrow::<UniqueViewMut<GameMode>>().unwrap();
                                    *gamemode = GameMode::Sim;
                                }
                                self.generate_map(1);
                                
                                new_runstate = RunState::MapGenAnimation
                            }
                            gui_menus::MainMenuSelection::LoadGame => {new_runstate = RunState::PreRun}
                            gui_menus::MainMenuSelection::Exit => {::std::process::exit(0)}
                        }
                    }
                }
            }
            RunState::SaveGame => {
                /*
                let data = serde_json::to_string(&*self.resources.get::<Map>().unwrap()).unwrap();
                println!("{}", data);
    
                let c: Context;
                let mut writer = Vec::with_capacity(128);
                let s = serde_json::Serializer::new(writer);
                hecs::serialize::row::serialize(&self.world, &mut c, s);

                for (id, _s) in self.world.query_mut::<&SerializeMe>() {
                    println!("{:?}", id);
                }
                */
                println!("Saving game... TODO");
                self.game_over_cleanup();
                new_runstate = RunState::MainMenu{menu_selection: gui_menus::MainMenuSelection::LoadGame};
            }
            RunState::NextLevel => {
                self.next_level();
                new_runstate = RunState::PreRun;
            }
            RunState::GameOver => {
                let result = gui_menus::game_over(ctx);
                match result {
                    gui_menus::GameOverResult::NoSelection => {}
                    gui_menus::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        new_runstate = RunState::MainMenu {menu_selection: gui_menus::MainMenuSelection::Roguelike};
                    }
                }
            }
            RunState::MapGenAnimation => {
                if !SHOW_MAPGEN_ANIMATION {
                    new_runstate = RunState::PreRun;
                }else{
                    ctx.cls();
                    // todo bring back mapgen rendering
                    // map::draw_map(&self.mapgen_data.history[self.mapgen_data.index], ctx);
                    camera::render_camera(self, ctx);

                    self.mapgen_data.timer += ctx.frame_time_ms;
                    if self.mapgen_data.timer > MAPGEN_FRAME_TIME {
                        self.mapgen_data.timer = 0.0;
                        self.mapgen_data.index += 1;
                        if self.mapgen_data.index >= self.mapgen_data.history.len() {
                            new_runstate = RunState::PreRun;
                        }
                    }
                }       
            }
        }

        // self.resources.insert::<RunState>(new_runstate).unwrap();
        self.world.add_unique(new_runstate);

        self.world.run(system_cleanup::run_cleanup_system);
        // system_cleanup::run_cleanup_system(&mut self.world, &mut self.resources);
    }
}

fn main() -> rltk::BError {
    println!("=========================");
    println!("==== Start game =========");
    println!("=========================");

    let xscaled = (WINDOWWIDTH  as f32 / SCALE) as i32;
    let yscaled = (WINDOWHEIGHT as f32 / SCALE) as i32;
        
    let context = RltkBuilder::simple(WINDOWWIDTH, WINDOWHEIGHT).unwrap()
        .with_tile_dimensions(TILE_SIZE, TILE_SIZE)
        .with_title("Roguelike")
        .with_fps_cap(30.0)
        .with_fitscreen(true)
        .with_simple_console(xscaled, yscaled, "terminal8x8.png") // map layer
        .build()?;

    let gs = State {
        world: World::new(),
        // resources: Resources::default(),
        mapgen_data: MapGenData{history: Vec::new(), timer: 0.0, index: 0},
        autorun: false,
        wait_frames: 0
    };

    gs.world.add_unique(Map::new(1, TileType::Wall));
    gs.world.add_unique(PPoint(Point::new(0, 0)));
    gs.world.add_unique(Turn(0));
    gs.world.add_unique(RNG(rltk::RandomNumberGenerator::new()));

    let player_id = gs.world.run(|mut store: AllStoragesViewMut|{entity_factory::player(&mut store, (0, 0))});//entity_factory::player(&mut gs.world, (0, 0));
    gs.world.add_unique(PlayerID(player_id));

    gs.world.add_unique(GameMode::NotSelected);
    gs.world.add_unique(RunState::MainMenu{menu_selection: gui_menus::MainMenuSelection::Roguelike});
    gs.world.add_unique(gamelog::GameLog{messages: vec!["Welcome to the roguelike!".to_string()]});
    gs.world.add_unique(system_particle::ParticleBuilder::new());
    gs.world.add_unique(FrameTime(0.));

    rltk::main_loop(context, gs)
}
