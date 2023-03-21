use resources::Resources;
use rltk::RandomNumberGenerator;
use shipyard::{EntityId, World};
use crate::map::TileType;
use crate::RunState;
use crate::components::{CombatStats, Fire, Position};
use crate::effects::{EffectType, Targets, add_effect};
use crate::Map;

pub const NEW_FIRE_TURNS: i32 = 10;

pub fn run_fire_system(world: &mut World, res: &mut Resources) {
    let runstate: &RunState = &res.get::<RunState>().unwrap();
    if *runstate != RunState::PlayerTurn { return; }

    let mut map = res.get_mut::<Map>().unwrap();
    let mut rng = RandomNumberGenerator::new();

    // damage all entities on fire. If they are standing somewhere flammable, ignite it
    for (id, (_stats, _fire, pos)) in world.query::<(&mut CombatStats, &mut Fire, &mut Position)>().iter() {
        add_effect(
            None,
            EffectType::Damage{ amount: 1 },
            Targets::Single{ target: id }
        );

        for pos in pos.ps.iter() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.is_flammable(idx) && map.fire_turns[idx] == 0 {
                map.fire_turns[idx] = NEW_FIRE_TURNS;
            }
        }
    }

    // reduce fire turns and remove expired fire components
    let mut to_remove_fire: Vec<EntityId> = Vec::new();

    for (id, fire) in world.query::<&mut Fire>().iter() {
        fire.turns -= 1;

        if fire.turns <= 0 {
            to_remove_fire.push(id);
        }
    }

    for id in to_remove_fire.iter() {
        world.remove_one::<Fire>(*id).unwrap();
    }

    // reduce fire turns on tiles
    for idx in 0..(map.width*map.height) as usize {
        if map.fire_turns[idx] > 0 {
            map.fire_turns[idx] -= 1;

            if map.fire_turns[idx] == 0 && map.is_flammable(idx) {
                map.tiles[idx] = TileType::Dirt;
            }

            // light entities on this tile on fire
            for e in map.tile_content[idx].iter() {
                add_effect(
                    None,
                    EffectType::Fire { turns: NEW_FIRE_TURNS },
                    Targets::Single{ target: *e }
                );
            }

            // Chance to spread to nearby tiles
            let (x, y) = map.idx_xy(idx);
            for dx in -1..=1 {
                for dy in -1..=1 {
                    let (nx, ny) = (x+dx, y+dy);
                    if map.in_bounds(nx, ny) {
                        let idx = map.xy_idx(nx, ny);
                        if map.fire_turns[idx] == 0 && map.is_flammable(idx) && rng.range(0, 10) == 0 {
                            map.fire_turns[idx] = NEW_FIRE_TURNS;
                        }
                    }
                }
            }
        }
    }
}
