use engine::components::{Ranged, WantsToDropItem, WantsToUseItem, WantsToUnequipItem};
use engine::gui::{gui_menus, self};
use engine::systems::{system_cleanup, system_particle};
use engine::utils::{PlayerID, Turn};
use engine::{ gamelog, EngineController, GameMode, RunState, effects, GameTools};
use engine::{map_builders::MapGenData, State, SCALE, TILE_SIZE, WINDOWHEIGHT, WINDOWWIDTH};
use render::camera;
use rltk::{ Rltk, RltkBuilder};
use shipyard::{ UniqueViewMut, World, UniqueView, EntityId, ViewMut, Get};

pub mod render;

pub struct Game {}

impl Game {
    fn new() -> Box<dyn EngineController> {
        Box::new(Game {})
    }
}

impl EngineController for Game {
    fn render(&self, gs: &State, ctx: &mut Rltk) {
        let new_runstate = *gs.world.borrow::<UniqueViewMut<RunState>>().unwrap();

        match new_runstate {
            RunState::MainMenu { .. } => {}
            RunState::GameOver => {}
            _ => {
                camera::render_camera(gs, ctx);
                render::draw_gui(gs, ctx);
            }
        }
    }

    fn update(&self, world: &mut World, ctx: &mut Rltk) {
        // dbg!("update");



        let mut new_runstate = *world.borrow::<UniqueViewMut<RunState>>().unwrap();
        let player_id = world.borrow::<UniqueView<PlayerID>>().unwrap().0;
        // dbg!(new_runstate);

        world.run(system_particle::update_particles);
        world.run(effects::run_effects_queue);

        match new_runstate {
            RunState::PreRun => {
                GameTools::run_systems(world, new_runstate);
                new_runstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                new_runstate = input_handler::handle_input(&world, ctx);
            }
            RunState::PlayerTurn => {
                GameTools::run_systems(world, new_runstate);
                new_runstate = RunState::AiTurn;
            }
            RunState::AiTurn => {
                {
                    let mut turn = world.borrow::<UniqueViewMut<Turn>>().unwrap();
                    turn.0 += 1;
                }
                GameTools::run_systems(world, new_runstate);
                new_runstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui_menus::show_inventory(world, ctx);
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
                let result = gui_menus::show_item_actions(&world, item, ctx);
                match result {
                    gui_menus::ItemActionSelection::NoSelection => {}
                    gui_menus::ItemActionSelection::Used => {
                        let mut to_add_wants_use_item: Vec<EntityId> = Vec::new();
                        {
                            let vranged = world.borrow::<ViewMut<Ranged>>().unwrap();
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
                            world
                                .add_component(*id, WantsToUseItem { item, target: None });
                        }
                    }
                    gui_menus::ItemActionSelection::Dropped => {
                        let player_id = world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
                        world
                            .add_component(player_id, WantsToDropItem { item });
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui_menus::ItemActionSelection::Unequipped => {
                        let player_id = world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
                        world
                            .add_component(player_id, WantsToUnequipItem { item });
                        new_runstate = RunState::PlayerTurn;
                    }
                    gui_menus::ItemActionSelection::Cancel => {
                        new_runstate = RunState::ShowInventory
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let res = gui::ranged_target(world, ctx, range);
                match res.0 {
                    gui::ItemMenuResult::Cancel => new_runstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let player_id = world.borrow::<UniqueViewMut<PlayerID>>().unwrap().0;
                        world.add_component(
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
                let result = gui_menus::main_menu(world, ctx);
                match result {
                    gui_menus::MainMenuResult::NoSelection { selected } => {
                        new_runstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    gui_menus::MainMenuResult::Selection { selected } => match selected {
                        gui_menus::MainMenuSelection::Roguelike => {
                            GameTools::set_game_mode(world, GameMode::RL);
                            GameTools::generate_map(world, 1);

                            new_runstate = RunState::MapGenAnimation
                        }
                        gui_menus::MainMenuSelection::Simulator => {
                            GameTools::set_game_mode(world, GameMode::Sim);
                            GameTools::generate_map(world, 1);

                            new_runstate = RunState::MapGenAnimation
                        }
                        gui_menus::MainMenuSelection::Exit => ::std::process::exit(0),
                    },
                }
            }
            RunState::EscPressed => {
                GameTools::reset_engine(world);
                new_runstate = RunState::MainMenu {
                    menu_selection: gui_menus::MainMenuSelection::Roguelike,
                };
            }
            RunState::NextLevel => {
                GameTools::next_level(world);
                new_runstate = RunState::PreRun;
            }
            RunState::GameOver => {
                let result = gui_menus::game_over(ctx);
                match result {
                    gui_menus::GameOverResult::NoSelection => {}
                    gui_menus::GameOverResult::QuitToMenu => {
                        GameTools::reset_engine(world);
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

        world.run(|mut rs: UniqueViewMut<RunState>| {
            *rs = new_runstate;
        });
        world.run(system_cleanup::run_cleanup_system);
    }

    fn start(&self, world: &mut World) {
        GameTools::reset_engine(world);
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
        world: World::new(),
        // resources: Resources::default(),
        mapgen_data: MapGenData {
            history: Vec::new(),
            timer: 0.0,
            index: 0,
        },
        engine_controller: Game::new(),
        first_run: true,
    };

    rltk::main_loop(context, gs)
}