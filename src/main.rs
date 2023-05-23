use engine::gui::gui_menus;
use engine::systems::system_particle;
use engine::utils::{Turn, PPoint, RNG, FrameTime, AutoRun, PlayerID};
use engine::{MAPWIDTH, MAPHEIGHT, entity_factory, gamelog, GameMode, RunState};
use engine::{State, map_builders::MapGenData, WINDOWWIDTH, SCALE, WINDOWHEIGHT, TILE_SIZE};
use engine::map::{TileType, Map};
use rltk::{Point, RltkBuilder};
use shipyard::{World, AllStoragesViewMut};

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
        wait_frames: 0
    };

    gs.world.add_unique(Map::new(1, TileType::Wall, (MAPWIDTH as i32, MAPHEIGHT as i32)));
    gs.world.add_unique(PPoint(Point::new(0, 0)));
    gs.world.add_unique(Turn(0));
    gs.world.add_unique(RNG(rltk::RandomNumberGenerator::new()));

    let player_id = gs.world.run(|mut store: AllStoragesViewMut|{entity_factory::player(&mut store, (0, 0))});
    gs.world.add_unique(PlayerID(player_id));

    gs.world.add_unique(GameMode::NotSelected);
    gs.world.add_unique(RunState::MainMenu{menu_selection: gui_menus::MainMenuSelection::Roguelike});
    gs.world.add_unique(gamelog::GameLog{messages: vec![]});
    gs.world.add_unique(system_particle::ParticleBuilder::new());
    gs.world.add_unique(FrameTime(0.));
    gs.world.add_unique(AutoRun(false));

    rltk::main_loop(context, gs)
}
