use engine::components::{Ranged, WantsToDropItem, WantsToUnequipItem, WantsToUseItem};
use engine::systems::{system_cleanup, system_particle, system_visibility};
use engine::utils::{FrameTime, PlayerID, Turn};
use engine::{effects, gamelog, Engine, GameMode, GameSettings};
use engine::{map_builders::MapGenData, SCALE, TILE_SIZE};
use render::{camera, gui_menus};
use rltk::{GameState, Rltk, RltkBuilder, RGBA};
use shipyard::{EntityId, Get, UniqueView, UniqueViewMut, ViewMut, World};

use crate::game_modes::get_settings;

pub mod game_modes;
pub mod input_handler;
pub mod render;

pub const WINDOWWIDTH: usize = 160;
pub const WINDOWHEIGHT: usize = 80;

#[derive(Copy, Clone, PartialEq, Debug)]
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
    EscPressed,
    NextLevel,
    GameOver,
    MapGenAnimation,
}

pub struct State {
    pub engine: Engine,
    pub mapgen_data: MapGenData,
    pub state: RunState,
    pub settings: GameSettings,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        if self.engine.first_run {
            self.engine.first_run = false;

            self.engine.reset_engine(game_modes::get_settings(GameMode::RL));
        }

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
            let mut i = self.engine.world.borrow::<UniqueViewMut<FrameTime>>().unwrap();
            i.0 = ctx.frame_time_ms;
        }

        // self.engine_controller.update(&mut self.world, ctx);

        let mut new_runstate = self.state; //*self.world.borrow::<UniqueViewMut<RunState>>().unwrap();

        let player_id = self.engine.world.borrow::<UniqueView<PlayerID>>().unwrap().0;
        // dbg!(new_runstate);

        self.engine.world.run(system_particle::update_particles);
        self.engine.world.run(effects::run_effects_queue);

        match new_runstate {
            RunState::PreRun => {
                Engine::run_systems(
                    &mut self.engine.world,
                    new_runstate == RunState::PlayerTurn,
                    new_runstate == RunState::AiTurn,
                );
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_runstate = input_handler::handle_input(&self.engine.world, ctx);
            }
            RunState::PlayerTurn => {
                Engine::run_systems(
                    &mut self.engine.world,
                    new_runstate == RunState::PlayerTurn,
                    new_runstate == RunState::AiTurn,
                );
                new_runstate = RunState::AiTurn;
            }
            RunState::AiTurn => {
                {
                    let mut turn = self.engine.world.borrow::<UniqueViewMut<Turn>>().unwrap();
                    turn.0 += 1;
                }
                Engine::run_systems(
                    &mut self.engine.world,
                    new_runstate == RunState::PlayerTurn,
                    new_runstate == RunState::AiTurn,
                );
                new_runstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui_menus::show_inventory(&self.engine.world, ctx);
                match result.0 {
                    render::ItemMenuResult::NoResponse => {}
                    render::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    render::ItemMenuResult::Selected => {
                        new_runstate = RunState::ShowItemActions {
                            item: result.1.unwrap(),
                        }
                    }
                }
            }
            RunState::ShowItemActions { item } => {
                let result = gui_menus::show_item_actions(&self.engine.world, item, ctx);
                match result {
                    gui_menus::ItemActionSelection::NoSelection => {}
                    gui_menus::ItemActionSelection::Used => {
                        let mut to_add_wants_use_item: Vec<EntityId> = Vec::new();
                        {
                            let vranged = self.engine.world.borrow::<ViewMut<Ranged>>().unwrap();
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
                            self.engine
                                .world
                                .add_component(*id, WantsToUseItem { item, target: None });
                        }
                    }
                    gui_menus::ItemActionSelection::Dropped => {
                        let player_id = self
                            .engine
                            .world
                            .borrow::<UniqueViewMut<PlayerID>>()
                            .unwrap()
                            .0;
                        self.engine
                            .world
                            .add_component(player_id, WantsToDropItem { item });
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui_menus::ItemActionSelection::Unequipped => {
                        let player_id = self
                            .engine
                            .world
                            .borrow::<UniqueViewMut<PlayerID>>()
                            .unwrap()
                            .0;
                        self.engine
                            .world
                            .add_component(player_id, WantsToUnequipItem { item });
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui_menus::ItemActionSelection::Cancel => {
                        new_runstate = RunState::ShowInventory
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let res = render::ranged_target(&self.engine.world, ctx, range);
                match res.0 {
                    render::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    render::ItemMenuResult::NoResponse => {}
                    render::ItemMenuResult::Selected => {
                        let player_id = self
                            .engine
                            .world
                            .borrow::<UniqueViewMut<PlayerID>>()
                            .unwrap()
                            .0;
                        self.engine.world.add_component(
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
                let result = gui_menus::main_menu(ctx, new_runstate);
                match result {
                    gui_menus::MainMenuResult::NoSelection { selected } => {
                        new_runstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    gui_menus::MainMenuResult::Selection { selected } => match selected {
                        gui_menus::MainMenuSelection::Roguelike => {
                            self.engine.reset_engine(game_modes::get_settings(GameMode::RL));
                            new_runstate = RunState::MapGenAnimation
                        }
                        gui_menus::MainMenuSelection::Simulator => {
                            self.engine.reset_engine(game_modes::get_settings(GameMode::VillageSim));
                            new_runstate = RunState::MapGenAnimation
                        }
                        gui_menus::MainMenuSelection::OrcHalls => {
                            self.engine.reset_engine(game_modes::get_settings(GameMode::OrcHalls));
                            new_runstate = RunState::MapGenAnimation
                        }
                        gui_menus::MainMenuSelection::Exit => ::std::process::exit(0),
                    },
                }
            }
            RunState::EscPressed => {
                // Engine::reset_engine(world);
                new_runstate = RunState::MainMenu {
                    menu_selection: gui_menus::MainMenuSelection::Roguelike,
                };
            }
            RunState::NextLevel => {
                Engine::next_level(&mut self.engine.world);
                new_runstate = RunState::PreRun;
            }
            RunState::GameOver => {
                let result = gui_menus::game_over(ctx);
                match result {
                    gui_menus::GameOverResult::NoSelection => {}
                    gui_menus::GameOverResult::QuitToMenu => {
                        // Engine::reset_engine(world);
                        new_runstate = RunState::MainMenu {
                            menu_selection: gui_menus::MainMenuSelection::Roguelike,
                        };
                    }
                }
            }
            RunState::MapGenAnimation => {
                new_runstate = RunState::PreRun;

                // if !SHOW_MAPGEN_ANIMATION {
                //     new_runstate = RunState::PreRun;
                // } else {
                //     ctx.cls();
                //     // todo bring back mapgen rendering
                //     // map::draw_map(&gs.mapgen_data.history[gs.mapgen_data.index], ctx);
                //     gs.engine_controller.render(gs, ctx);

                //     gs.mapgen_data.timer += ctx.frame_time_ms;
                //     if gs.mapgen_data.timer > MAPGEN_FRAME_TIME {
                //         gs.mapgen_data.timer = 0.0;
                //         gs.mapgen_data.index += 1;
                //         if gs.mapgen_data.index >= gs.mapgen_data.history.len() {
                //             new_runstate = RunState::PreRun;
                //         }
                //     }
                // }
            }
        }

        self.state = new_runstate;

        self.engine
            .world
            .run(system_visibility::run_visibility_system);
        self.engine.world.run(system_cleanup::run_cleanup_system);

        //now render
        match self.state {
            RunState::MainMenu { .. } | RunState::EscPressed | RunState::GameOver => {}
            _ => {
                camera::render_game(&self.engine.world, ctx);
                render::draw_gui(&self.engine.world, ctx);
            }
        }
    }
}

fn main() -> rltk::BError {
    println!("=========================");
    println!("==== Start game =========");
    println!("=========================");

    let xscaled = (WINDOWWIDTH as f32 / SCALE) as i32;
    let yscaled = (WINDOWHEIGHT as f32 / SCALE) as i32;

    let context = RltkBuilder::simple(WINDOWWIDTH, WINDOWHEIGHT)
        .unwrap()
        .with_tile_dimensions(TILE_SIZE, TILE_SIZE)
        .with_title("Roguelike")
        .with_fps_cap(30.0)
        .with_fitscreen(true)
        .with_simple_console(xscaled, yscaled, "terminal8x8.png") // map layer
        .build()?;

    let gs = State {
        engine: Engine {
            world: World::new(),
            first_run: true,
        },
        mapgen_data: MapGenData {
            history: Vec::new(),
            timer: 0.0,
            index: 0,
        },
        state: RunState::MainMenu {
            menu_selection: gui_menus::MainMenuSelection::Roguelike,
        },
        settings: get_settings(GameMode::RL),
    };

    rltk::main_loop(context, gs)
}
