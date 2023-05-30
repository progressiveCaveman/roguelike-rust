use engine::{ gamelog, EngineController, GameMode, RunState};
use engine::{map_builders::MapGenData, State, SCALE, TILE_SIZE, WINDOWHEIGHT, WINDOWWIDTH};
use render::camera;
use rltk::{ Rltk, RltkBuilder};
use shipyard::{ UniqueViewMut, World};

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

    fn update(&self, gs: &State) {
        // dbg!("update");
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
        engine_controller: Game::new(),
        first_run: true,
    };

    rltk::main_loop(context, gs)
}