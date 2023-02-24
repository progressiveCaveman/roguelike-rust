#[macro_use]
extern crate lazy_static;

use map::TileType;
use rltk::{RGB, RGBA};
use rltk::{Rltk, GameState, RltkBuilder, Point};
use hecs::*;
use resources::Resources;

mod inventory_system;

pub mod gui;
use gui::gui_menus;
use gui::camera;

mod player;
mod map;
mod components;
mod movement;
mod rect;
mod gamelog;
mod input_handler;
mod entity_factory;
mod weighted_table;
mod dijkstra_utils;

pub mod map_builders;
use map_builders::MapGenData;

pub mod systems;
use systems::cleanup_system;
use systems::fire_system;
use systems::map_indexing_system;
use systems::melee_combat_system;
use systems::monster_ai_system;
use systems::particle_system;
use systems::visibility_system;
use systems::spawner_ai;

pub mod effects;

use components::{Position, WantsToUseItem, WantsToDropItem, Ranged, InBackpack, Player, Viewshed, Equipped, WantsToUnequipItem};
use map::Map;
use gamelog::GameLog;
use inventory_system::{drop_item, item_use, unequip_item};

// TODO move to a utils file
trait Scale {
    fn scale(&mut self, amount: f32);
}

impl Scale for RGB {
    fn scale(&mut self, amount: f32) {
        self.r *= amount;
        self.g *= amount;
        self.b *= amount;
    }
}

const SHOW_MAPGEN_ANIMATION: bool = true;
const MAPGEN_FRAME_TIME: f32 = 25.0;

const TILE_SIZE: usize = 10;
const MAPWIDTH: usize = 200;
const MAPHEIGHT: usize = 100;
const WINDOWWIDTH: usize = 160;
const WINDOWHEIGHT: usize = 100;
const SCALE: f32 = 1.;

#[derive(Copy, Clone, PartialEq)]
pub enum GameMode{
    NotSelected,
    Sim,
    RL,
}

#[derive(Copy, Clone, PartialEq)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    AiTurn,
    ShowInventory,
    ShowItemActions {item: Entity},
    ShowTargeting {range: i32, item: Entity},
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
    resources: Resources,
    mapgen_data: MapGenData, 
    autorun: bool,
    wait_frames: i32
}

impl State {
    fn run_systems(&mut self) {
        fire_system::fire(&mut self.world, &mut self.resources);
        visibility_system::visibility(&mut self.world, &mut self.resources);
        spawner_ai::run(&mut self.world, &mut self.resources);
        monster_ai_system::monster_ai(self);
        map_indexing_system::map_indexing(&mut self.world, &mut self.resources);
        melee_combat_system::melee_combat(&mut self.world, &mut self.resources);
        inventory_system::inventory(&mut self.world, &mut self.resources);
        drop_item(&mut self.world, &mut self.resources);
        unequip_item(&mut self.world, &mut self.resources);
        item_use(&mut self.world, &mut self.resources);
        effects::run_effects_queue(&mut self.world, &mut self.resources);
        particle_system::spawn_particles(&mut self.world, &mut self.resources);
    }

    fn entities_to_delete_on_level_change(&mut self) -> Vec<Entity> {
        let mut ids_to_delete: Vec<Entity> = Vec::new();
        let all_entities: Vec<Entity> = self.world.iter().map(|(id, _)| id).collect();

        let player_id = self.resources.get::<Entity>().unwrap();

        for id in all_entities {
            let mut to_delete = true;
            if let Ok(_p) =  self.world.get::<Player>(id) { to_delete = false; }
            
            if let Ok(backpack) = self.world.get::<InBackpack>(id) {
                if backpack.owner == *player_id { to_delete = false; }
            }

            if let Ok(equipped) = self.world.get::<Equipped>(id) {
                if equipped.owner == *player_id { to_delete = false; }
            }

            if to_delete { ids_to_delete.push(id); }
        }

        ids_to_delete
    }

    fn generate_map(&mut self, new_depth: i32) {

        // delete all entities
        let ids_to_delete = self.entities_to_delete_on_level_change();
        for id in ids_to_delete {
            self.world.despawn(id).unwrap();
        }

        self.mapgen_data.index = 0;
        self.mapgen_data.timer = 0.0;
        self.mapgen_data.history.clear();

        // get game mode
        let gamemode = *self.resources.get::<GameMode>().unwrap();

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
            let mut map = self.resources.get_mut::<Map>().unwrap();
            *map = map_builder.get_map();
            start_pos = map_builder.get_starting_position().ps.first().unwrap().clone();
        }

        // Spawn monsters and items
        map_builder.spawn_entities(&mut self.world, &mut self.resources);

        // Update player position
        let mut player_position = self.resources.get_mut::<Point>().unwrap();
        *player_position = Point::new(start_pos.x, start_pos.y);
        let player_id = self.resources.get::<Entity>().unwrap();
        let mut player_pos_comp = self.world.get_mut::<Position>(*player_id).unwrap();
        player_pos_comp.ps[0].x = start_pos.x;
        player_pos_comp.ps[0].y = start_pos.y;

        // Mark viewshed as dirty
        let player_vs = self.world.get_mut::<Viewshed>(*player_id);
        if let Ok(mut vs) = player_vs { vs.dirty = true; }
    }

    fn next_level(&mut self) {
        // Generate new map
        let current_depth;
        {
            let map = self.resources.get_mut::<Map>().unwrap();
            current_depth = map.depth;
        }
        self.generate_map(current_depth + 1);

        // Notify player
        let mut log = self.resources.get_mut::<GameLog>().unwrap();
        log.messages.push("You descend in the staircase".to_string());
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        self.world.clear();

        // Create player
        let player_id = entity_factory::player(&mut self.world, (0, 0));
        self.resources.insert(Point::new(0, 0));
        self.resources.insert(player_id);

        // Generate new map
        self.generate_map(1);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(0);
        ctx.cls();
        
        particle_system::update_particles(&mut self.world, &mut self.resources, ctx);

        let mut new_runstate: RunState = *self.resources.get::<RunState>().unwrap();

        match new_runstate {
            RunState::MainMenu{..} => {}
            RunState::GameOver => {}
            _ => {
                camera::render_camera(&self.world, &self.resources, ctx);
                gui::draw_gui(&self.world, &self.resources, ctx);
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
                    let mut turn = self.resources.get_mut::<i32>().unwrap();
                    *turn += 1;

                    let map = &mut self.resources.get_mut::<Map>().unwrap();
                    let gamemode = *self.resources.get::<GameMode>().unwrap();
                    if gamemode == GameMode::Sim{
                        map.refresh_influence_maps(self, *turn);
                    }
                }
                self.run_systems();
                new_runstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui_menus::show_inventory(&mut self.world, &mut self.resources, ctx);
                match result.0 {
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Cancel => { new_runstate = RunState::AwaitingInput }
                    gui::ItemMenuResult::Selected => {
                        new_runstate = RunState::ShowItemActions{ item: result.1.unwrap() }
                    }
                }
            }
            RunState::ShowItemActions{item} => {
                let result = gui_menus::show_item_actions(&mut self.world, &mut self.resources, item, ctx);
                match result {
                    gui_menus::ItemActionSelection::NoSelection => {}
                    gui_menus::ItemActionSelection::Used => {
                        let mut to_add_wants_use_item: Vec<Entity> = Vec::new();
                        {
                            let player_id = self.resources.get::<Entity>().unwrap();
                            let is_item_ranged = self.world.get::<Ranged>(item);
                            match is_item_ranged {
                                Ok(is_item_ranged) => {
                                    new_runstate = RunState::ShowTargeting{range:is_item_ranged.range, item};
                                }
                                Err(_) => {
                                    to_add_wants_use_item.push(*player_id);
                                    new_runstate = RunState::PlayerTurn;
                                }
                            }
                        }

                        for id in to_add_wants_use_item.iter() {
                            self.world.insert_one(*id, WantsToUseItem {item, target: None}).unwrap();
                        }
                    }
                    gui_menus::ItemActionSelection::Dropped => {
                        let player_id = self.resources.get::<Entity>().unwrap();
                        self.world.insert_one(*player_id, WantsToDropItem {item}).unwrap();
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui_menus::ItemActionSelection::Unequipped => {
                        let player_id = self.resources.get::<Entity>().unwrap();
                        self.world.insert_one(*player_id, WantsToUnequipItem{item}).unwrap();
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui_menus::ItemActionSelection::Cancel => { new_runstate = RunState::ShowInventory}
                }
            }
            RunState::ShowTargeting{range, item} => {
                let res = gui::ranged_target(&mut self.world, &mut self.resources, ctx, range);
                match res.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {},
                    gui::ItemMenuResult::Selected => {
                        let player_id = self.resources.get::<Entity>().unwrap();
                        self.world.insert_one(*player_id, WantsToUseItem{item, target: res.1}).unwrap();
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MainMenu{..} => {
                let result = gui_menus::main_menu(&mut self.world, &mut self.resources, ctx);
                match result {
                    gui_menus::MainMenuResult::NoSelection{selected} => {new_runstate = RunState::MainMenu{menu_selection: selected}}
                    gui_menus::MainMenuResult::Selection{selected} => {
                        match selected {
                            gui_menus::MainMenuSelection::Roguelike => {
                                {
                                    let mut gamemode = self.resources.get_mut::<GameMode>().unwrap();
                                    *gamemode = GameMode::RL;
                                }
                                self.generate_map(1);

                                new_runstate = RunState::MapGenAnimation
                            }
                            gui_menus::MainMenuSelection::Simulator => {
                                {
                                    let mut gamemode = self.resources.get_mut::<GameMode>().unwrap();
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
                    // map::draw_map(&self.mapgen_data.history[self.mapgen_data.index], ctx);
                    camera::render_camera(&self.world, &self.resources, ctx);

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

        self.resources.insert::<RunState>(new_runstate).unwrap();

        cleanup_system::delete_the_dead(&mut self.world, &mut self.resources);
    }
}

fn main() -> rltk::BError {
    // let context = RltkBuilder::new()
    //     .with_title("roguelike")
    //     .with_fps_cap(30.0)
    //     .with_dimensions(WINDOWWIDTH, WINDOWHEIGHT)
    //     .with_tile_dimensions(32, 32)
    //     .with_resource_path("resources/")
    //     .with_font("dungeonfont.png", 32, 32)
    //     .with_font("terminal8x8.png", 8, 8)
    //     .with_simple_console(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
    //     .with_simple_console_no_bg(DISPLAY_WIDTH, DISPLAY_HEIGHT, "dungeonfont.png")
    //     .with_simple_console_no_bg(SCREEN_WIDTH*2, SCREEN_HEIGHT*2, "terminal8x8.png")
    //     .build()?;

    let xscaled = (WINDOWWIDTH  as f32 / SCALE) as i32;
    let yscaled = (WINDOWHEIGHT as f32 / SCALE) as i32;
        
    let context = RltkBuilder::simple(WINDOWWIDTH, WINDOWHEIGHT).unwrap()
        .with_tile_dimensions(TILE_SIZE, TILE_SIZE)
        .with_title("Roguelike")
        .with_fps_cap(30.0)
        // .with_automatic_console_resize(true)
        .with_fitscreen(true)
        .with_simple_console_no_bg(xscaled, yscaled, "terminal8x8.png")
        // .with_tile_dimensions(8, 8)
        // .with_simple_console_no_bg(MAPWIDTH, MAPHEIGHT, "terminal8x8.png")
        // .with_tile_dimensions(10, 10)
        // .with_simple_console_no_bg(MAPWIDTH, MAPHEIGHT, "terminal8x8.png")
        // .with_tile_dimensions(12, 12)
        .build()?;

    let mut gs = State {
        world: World::new(),
        resources: Resources::default(),
        mapgen_data: MapGenData{history: Vec::new(), timer: 0.0, index: 0},
        autorun: false,
        wait_frames: 0
    };

    gs.resources.insert(Map::new(1, TileType::Wall));
    gs.resources.insert(Point::new(0, 0));
    gs.resources.insert(0);
    gs.resources.insert(rltk::RandomNumberGenerator::new());

    let player_id = entity_factory::player(&mut gs.world, (0, 0));
    gs.resources.insert(player_id);

    gs.resources.insert(GameMode::NotSelected);
    gs.resources.insert(RunState::MainMenu{menu_selection: gui_menus::MainMenuSelection::Roguelike});
    gs.resources.insert(gamelog::GameLog{messages: vec!["Welcome to the roguelike!".to_string()]});
    gs.resources.insert(particle_system::ParticleBuilder::new());

    rltk::main_loop(context, gs)
}
