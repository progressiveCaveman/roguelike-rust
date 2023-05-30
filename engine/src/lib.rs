#[macro_use]
extern crate lazy_static;

use components::{
    Equipped, InBackpack, Player, Position, Ranged, Viewshed, WantsToDropItem, WantsToUnequipItem,
    WantsToUseItem,
};
use gamelog::GameLog;
use item_system::{run_drop_item_system, run_item_use_system, run_unequip_item_system};
use map::Map;
use rltk::RGBA;
use rltk::{GameState, Point, Rltk};

mod item_system;

pub mod ai;

pub mod gui;
use gui::gui_menus;

pub mod components;
pub mod entity_factory;
pub mod gamelog;
pub mod input_handler;
pub mod map;
pub mod player;
pub mod rect;
pub mod utils;
pub mod weighted_table;

pub mod map_builders;
use map_builders::MapGenData;

pub mod systems;
use shipyard::{
    AllStoragesViewMut, EntitiesView, EntityId, Get, Unique, UniqueView, UniqueViewMut, View,
    ViewMut, World,
};
use systems::{
    system_ai_fish, system_ai_monster, system_ai_spawner, system_ai_villager, system_cleanup,
    system_dissasemble, system_fire, system_map_indexing, system_melee_combat, system_particle,
    system_pathfinding, system_visibility,
};
use utils::{AutoRun, FrameTime, PPoint, PlayerID, Turn};

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

#[derive(Copy, Clone, PartialEq, Unique, Debug)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    AiTurn,
    ShowInventory,
    ShowItemActions {
        item: EntityId,
    },
    ShowTargeting {
        range: i32,
        item: EntityId,
    },
    MainMenu {
        menu_selection: gui_menus::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
    GameOver,
    MapGenAnimation,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum RenderOrder {
    Items,
    NPC,
    Player,
    Particle,
}

pub trait EngineController: 'static {
    fn render(&self, gs: &State, ctx: &mut Rltk);
    fn update(&self, gs: &State);
}

pub struct State {
    pub world: World,
    pub mapgen_data: MapGenData,
    pub wait_frames: i32,
    pub engine_controller: Box<dyn EngineController>,
}

impl State {
    fn run_systems(&mut self, runstate: RunState) {
        if runstate == RunState::PlayerTurn {
            self.world.run(system_fire::run_fire_system);
        }
        self.world.run(system_visibility::run_visibility_system);

        self.world.run(effects::run_effects_queue);

        if runstate == RunState::AiTurn {
            self.world.run(system_pathfinding::run_pathfinding_system);
            self.world.run(system_ai_spawner::run_spawner_system);
            self.world.run(system_ai_fish::run_fish_ai);
            self.world.run(system_ai_villager::run_villager_ai_system);
            self.world.run(system_ai_monster::run_monster_ai_system);
            // system_ai_monster::run_monster_ai_system(self);
        }

        self.world.run(effects::run_effects_queue);

        self.world.run(system_map_indexing::run_map_indexing_system);

        self.world.run(system_melee_combat::run_melee_combat_system);
        self.world.run(item_system::run_inventory_system);
        self.world.run(system_dissasemble::run_dissasemble_system);
        self.world.run(run_drop_item_system);
        self.world.run(run_unequip_item_system);
        self.world.run(run_item_use_system);
        self.world.run(system_particle::spawn_particles);

        self.world.run(effects::run_effects_queue);
        self.world.run(system_map_indexing::run_map_indexing_system);
    }

    fn entities_to_delete_on_level_change(&mut self) -> Vec<EntityId> {
        let mut ids_to_delete: Vec<EntityId> = Vec::new();

        let entities = self.world.borrow::<EntitiesView>().unwrap();
        let player_id = self.world.borrow::<UniqueView<PlayerID>>().unwrap().0;

        let vplayer = self.world.borrow::<View<Player>>().unwrap();
        let vpack = self.world.borrow::<View<InBackpack>>().unwrap();
        let vequipped = self.world.borrow::<View<Equipped>>().unwrap();

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
        let gamemode = *self.world.borrow::<UniqueView<GameMode>>().unwrap();

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

        self.mapgen_data.history = map_builder.get_map_history();

        let start_pos;
        {
            let mut map = self.world.borrow::<UniqueViewMut<Map>>().unwrap();
            *map = map_builder.get_map();
            start_pos = map_builder
                .get_starting_position()
                .ps
                .first()
                .unwrap()
                .clone();
        }

        // Spawn monsters and items
        map_builder.spawn_entities(&mut self.world);

        // Update player position
        self.world.run(
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
        log.messages
            .push("You descend in the staircase".to_string());
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        self.world.clear();

        // Create player
        let player_id = self
            .world
            .run(|mut store: AllStoragesViewMut| entity_factory::player(&mut store, (0, 0)));
        self.world.add_unique(PPoint(Point::new(0, 0)));
        self.world.add_unique(PlayerID(player_id));

        // Generate new map
        self.generate_map(1);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.set_active_console(1);
        // write transparent bg
        let (x, y) = ctx.get_char_size();
        for ix in 0..x {
            for iy in 0..y {
                ctx.set(
                    ix,
                    iy,
                    RGBA::from_u8(0, 0, 0, 0),
                    RGBA::from_u8(0, 0, 0, 0),
                    32,
                )
            }
        }

        ctx.set_active_console(0);
        ctx.cls();

        {
            let mut i = self.world.borrow::<UniqueViewMut<FrameTime>>().unwrap();
            i.0 = ctx.frame_time_ms;
        }

        self.engine_controller.update(self);

        self.engine_controller.render(self, ctx);

        let mut new_runstate = *self.world.borrow::<UniqueViewMut<RunState>>().unwrap();
        // dbg!(new_runstate);

        self.world.run(system_particle::update_particles);
        self.world.run(effects::run_effects_queue);

        match new_runstate {
            RunState::PreRun => {
                self.run_systems(new_runstate);
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_runstate = input_handler::handle_input(self, ctx);

                let autorun = self.world.borrow::<UniqueView<AutoRun>>().unwrap().clone();

                if new_runstate == RunState::AwaitingInput && autorun.0 {
                    self.wait_frames += 1;
                    if self.wait_frames >= 10 {
                        self.wait_frames = 0;
                        new_runstate = RunState::PlayerTurn
                    }
                }
            }
            RunState::PlayerTurn => {
                self.run_systems(new_runstate);
                new_runstate = RunState::AiTurn;
            }
            RunState::AiTurn => {
                {
                    let mut turn = self.world.borrow::<UniqueViewMut<Turn>>().unwrap();
                    turn.0 += 1;
                }
                self.run_systems(new_runstate);
                new_runstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui_menus::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::Selected => {
                        new_runstate = RunState::ShowItemActions {
                            item: result.1.unwrap(),
                        }
                    }
                }
            }
            RunState::ShowItemActions { item } => {
                let result = gui_menus::show_item_actions(&mut self.world, item, ctx);
                match result {
                    gui_menus::ItemActionSelection::NoSelection => {}
                    gui_menus::ItemActionSelection::Used => {
                        let mut to_add_wants_use_item: Vec<EntityId> = Vec::new();
                        {
                            let player_id =
                                self.world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
                            let vranged = self.world.borrow::<ViewMut<Ranged>>().unwrap();
                            match vranged.get(item) {
                                Ok(is_item_ranged) => {
                                    new_runstate = RunState::ShowTargeting {
                                        range: is_item_ranged.range,
                                        item,
                                    };
                                }
                                Err(_) => {
                                    to_add_wants_use_item.push(player_id);
                                    new_runstate = RunState::PlayerTurn;
                                }
                            }
                        }

                        for id in to_add_wants_use_item.iter() {
                            self.world
                                .add_component(*id, WantsToUseItem { item, target: None });
                        }
                    }
                    gui_menus::ItemActionSelection::Dropped => {
                        let player_id = self.world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
                        self.world
                            .add_component(player_id, WantsToDropItem { item });
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui_menus::ItemActionSelection::Unequipped => {
                        let player_id = self.world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
                        self.world
                            .add_component(player_id, WantsToUnequipItem { item });
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui_menus::ItemActionSelection::Cancel => {
                        new_runstate = RunState::ShowInventory
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let res = gui::ranged_target(self, ctx, range);
                match res.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let player_id = self.world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
                        self.world.add_component(
                            player_id,
                            WantsToUseItem {
                                item,
                                target: res.1,
                            },
                        );
                        new_runstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let result = gui_menus::main_menu(self, ctx);
                match result {
                    gui_menus::MainMenuResult::NoSelection { selected } => {
                        new_runstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    gui_menus::MainMenuResult::Selection { selected } => match selected {
                        gui_menus::MainMenuSelection::Roguelike => {
                            {
                                let mut gamemode =
                                    self.world.borrow::<UniqueViewMut<GameMode>>().unwrap();
                                *gamemode = GameMode::RL;
                            }
                            self.generate_map(1);

                            new_runstate = RunState::MapGenAnimation
                        }
                        gui_menus::MainMenuSelection::Simulator => {
                            {
                                let mut gamemode =
                                    self.world.borrow::<UniqueViewMut<GameMode>>().unwrap();
                                *gamemode = GameMode::Sim;
                            }
                            self.generate_map(1);

                            new_runstate = RunState::MapGenAnimation
                        }
                        gui_menus::MainMenuSelection::LoadGame => new_runstate = RunState::PreRun,
                        gui_menus::MainMenuSelection::Exit => ::std::process::exit(0),
                    },
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
                new_runstate = RunState::MainMenu {
                    menu_selection: gui_menus::MainMenuSelection::LoadGame,
                };
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
                        new_runstate = RunState::MainMenu {
                            menu_selection: gui_menus::MainMenuSelection::Roguelike,
                        };
                    }
                }
            }
            RunState::MapGenAnimation => {
                if !SHOW_MAPGEN_ANIMATION {
                    new_runstate = RunState::PreRun;
                } else {
                    ctx.cls();
                    // todo bring back mapgen rendering
                    // map::draw_map(&self.mapgen_data.history[self.mapgen_data.index], ctx);
                    self.engine_controller.render(self, ctx);

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

        self.world.run(|mut rs: UniqueViewMut<RunState>| {
            *rs = new_runstate;
        });
        self.world.run(system_cleanup::run_cleanup_system);
    }
}
