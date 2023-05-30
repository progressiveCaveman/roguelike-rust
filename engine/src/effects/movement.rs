use rltk::{DijkstraMap, Point};
use shipyard::{AddComponent, Get, UniqueView, UniqueViewMut, View, ViewMut, World};

use super::*;
use crate::{
    components::{
        BlocksTile, CombatStats, Fire, LocomotionType, Locomotive, Player, Position,
        SpatialKnowledge, Viewshed, WantsToAttack,
    },
    map::{Map, TileType},
    utils::{dijkstra_backtrace, normalize, point_plus, PPoint},
    GameMode,
};

pub fn try_move_or_attack(gs: &mut State, effect: &EffectSpawner, attack: bool) {
    let mut map = gs.world.borrow::<UniqueViewMut<Map>>().unwrap();
    let mode = gs.world.borrow::<UniqueView<GameMode>>().unwrap();

    let mut vpos = gs.world.borrow::<ViewMut<Position>>().unwrap();
    let mut vvs = gs.world.borrow::<ViewMut<Viewshed>>().unwrap();
    let mut vwantsattack = gs.world.borrow::<ViewMut<WantsToAttack>>().unwrap();

    let entity = effect.creator.unwrap();
    let is_player = gs.world.run(|vplayer: View<Player>| {
        return vplayer.get(entity).is_ok();
    });

    let tile_idx = if let EffectType::Move { tile_idx } = effect.effect_type {
        tile_idx
    } else if let EffectType::MoveOrAttack { tile_idx } = effect.effect_type {
        tile_idx
    } else {
        0
    };

    if let Ok(pos) = (&mut vpos).get(entity) {
        let tp = map.idx_point(tile_idx);
        let dp = Point {
            x: normalize(tp.x - pos.ps[0].x),
            y: normalize(tp.y - pos.ps[0].y),
        };

        let canmove = can_move(&gs.world, &map, entity, &pos, dp);

        // In Sim mode, player is basically just a camera object
        let mut is_camera = false;
        if is_player && *mode == GameMode::Sim {
            is_camera = true;
            dbg!("is camera");
        }

        if !is_camera && attack {
            // dbg!("testing attack");
            if let Some(target) = get_target(&gs.world, &map, entity, &pos, dp) {
                // dbg!("found target");
                vwantsattack.add_component_unchecked(entity, WantsToAttack { target });
            }
        }

        // do movement
        if is_camera || canmove {
            // dbg!("can move");
            if let Ok(mut vs) = (&mut vvs).get(entity) {
                vs.dirty = true;
            }

            // for pos in pos.ps.iter_mut() {
            for i in 0..pos.ps.len() {
                let oldidx = map.point_idx(pos.ps[i]);

                pos.ps[i] = point_plus(pos.ps[i], dp);

                let idx = map.point_idx(pos.ps[i]);
                map.blocked[oldidx] = false;
                map.blocked[idx] = true;
            }

            // If this is a player, change the position in resources according to first in pos.ps
            if is_player {
                let mut ppos = gs.world.borrow::<UniqueViewMut<PPoint>>().unwrap();
                *ppos = PPoint(pos.ps[0]);
            }

            return;
        }
        // }
    }
}

pub fn autoexplore(gs: &mut State, effect: &EffectSpawner) {
    if let (Some(entity), EffectType::Explore {}) = (effect.creator, effect.effect_type.clone()) {
        // TODO Check for adjacent enemies and attack them

        // Use djikstras to find nearest unexplored tile
        let mut target = (0 as usize, std::f32::MAX); // tile_idx, distance
        {
            let dijkstra_map: DijkstraMap;
            let map = &mut gs.world.borrow::<UniqueViewMut<Map>>().unwrap();

            let vpos = gs.world.borrow::<View<Position>>().unwrap();
            let vspace = gs.world.borrow::<View<SpatialKnowledge>>().unwrap();

            let e_pos = if let Ok(pos) = vpos.get(entity) {
                pos
            } else {
                dbg!("No position found");
                return;
            };

            let e_space = if let Ok(space) = vspace.get(entity) {
                space
            } else {
                dbg!("Entity doesn't have a concept of space");
                return;
            };

            let e_idx = map.point_idx(e_pos.any_point());

            let starts: Vec<usize> = e_pos.idxes(map);
            dijkstra_map = rltk::DijkstraMap::new(map.width, map.height, &starts, &**map, 800.0);
            for (i, tile) in map.tiles.iter().enumerate() {
                if *tile != TileType::Wall && !e_space.tiles.contains_key(&i) {
                    let distance_to_start = dijkstra_map.map[i];

                    if distance_to_start < target.1 {
                        target = (i, distance_to_start)
                    }
                }
            }

            if target.1 == std::f32::MAX {
                // log.messages.push(format!("No tiles left to explore"));
                return;
            }

            // log.messages.push(format!("Closest unexplored tile is {} steps away", target.1));

            map.dijkstra_map = dijkstra_map.map.clone();

            // We have a target tile. Now follow the path up the chain
            let t = dijkstra_backtrace(dijkstra_map, map, e_idx, target.0);
            target = (t, 1.0);
        }

        // Send a move command
        // let dx: i32;
        // let dy: i32;
        // {
        //     dx = target.0.x - entity_point.x;
        //     dy = target.0.y - entity_point.y;
        // }

        // try_move_entity(entity, target.0, gs);

        let effect = &EffectSpawner {
            creator: effect.creator,
            effect_type: EffectType::Move { tile_idx: target.0 },
        };

        try_move_or_attack(gs, effect, true);
    }
}

pub fn skip_turn(gs: &mut State, effect: &EffectSpawner) {
    let mut vstats = gs.world.borrow::<ViewMut<CombatStats>>().unwrap();
    let vfire = gs.world.borrow::<View<Fire>>().unwrap();

    if let Some(id) = effect.creator {
        if let Ok(stats) = (&mut vstats).get(id) {
            if let Err(_) = vfire.get(id) {
                stats.hp = i32::min(stats.hp + stats.regen_rate, stats.max_hp);
            }
        }
    }
}

pub fn can_move(world: &World, map: &Map, entity: EntityId, pos: &Position, dp: Point) -> bool {
    let vloco = world.borrow::<View<Locomotive>>().unwrap();
    let vblocks = world.borrow::<View<BlocksTile>>().unwrap();

    if let Ok(loco) = vloco.get(entity) {
        for pos in pos.ps.iter() {
            // check for tiles that block
            let dest_idx = map.xy_idx(pos.x + dp.x, pos.y + dp.y);
            // let dest_idx = map.point_idx(tp);
            if loco.mtype == LocomotionType::Ground && map.blocks_movement(dest_idx) {
                return false;
            }

            if loco.mtype == LocomotionType::Water && map.tiles[dest_idx] != TileType::Water {
                return false;
            }

            // dbg!(1);
            // check for entities that block
            for potential_target in map.tile_content[dest_idx].iter() {
                if *potential_target == entity {
                    continue;
                }

                // dbg!(2);
                if vblocks.get(*potential_target).is_ok() {
                    return false;
                }
            }
        }

        return true;
    } else {
        dbg!("no locomotion");
    }

    return false;
}

// checks for entities with combat stats on block
pub fn get_target(
    world: &World,
    map: &Map,
    entity: EntityId,
    pos: &Position,
    dp: Point,
) -> Option<EntityId> {
    let vstats = world.borrow::<View<CombatStats>>().unwrap();

    // check for combat stats on entity
    if let Err(_) = vstats.get(entity) {
        // dbg!("No stats found");
        return None;
    }

    for pos in pos.ps.iter() {
        let dest_idx = map.xy_idx(pos.x + dp.x, pos.y + dp.y);
        // let dest_idx = map.point_idx(tp);
        for potential_target in map.tile_content[dest_idx].iter() {
            if *potential_target == entity {
                continue;
            }

            // dbg!("does enetiy have stats");
            match vstats.get(*potential_target) {
                Ok(_cs) => return Some(*potential_target),
                Err(_e) => {}
            }
        }
    }

    None
}
