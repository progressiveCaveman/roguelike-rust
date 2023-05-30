use engine::gui::gui_menus;
use engine::map::{Map, TileType};
use engine::systems::system_particle;
use engine::utils::{AutoRun, FrameTime, PPoint, PlayerID, Turn, RNG};
use engine::{entity_factory, gamelog, GameMode, EngineController, RunState, MAPHEIGHT, MAPWIDTH};
use engine::{map_builders::MapGenData, State, SCALE, TILE_SIZE, WINDOWHEIGHT, WINDOWWIDTH};
use render::camera;
use rltk::{Point, Rltk, RltkBuilder};
use shipyard::{AllStoragesViewMut, UniqueViewMut, World};

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
        wait_frames: 0,
        renderer: Game::new(),
    };

    gs.world.add_unique(Map::new(
        1,
        TileType::Wall,
        (MAPWIDTH as i32, MAPHEIGHT as i32),
    ));
    gs.world.add_unique(PPoint(Point::new(0, 0)));
    gs.world.add_unique(Turn(0));
    gs.world.add_unique(RNG(rltk::RandomNumberGenerator::new()));

    let player_id = gs
        .world
        .run(|mut store: AllStoragesViewMut| entity_factory::player(&mut store, (0, 0)));
    gs.world.add_unique(PlayerID(player_id));

    gs.world.add_unique(GameMode::NotSelected);
    gs.world.add_unique(RunState::MainMenu {
        menu_selection: gui_menus::MainMenuSelection::Roguelike,
    });
    gs.world.add_unique(gamelog::GameLog { messages: vec![] });
    gs.world.add_unique(system_particle::ParticleBuilder::new());
    gs.world.add_unique(FrameTime(0.));
    gs.world.add_unique(AutoRun(false));

    rltk::main_loop(context, gs)
}
