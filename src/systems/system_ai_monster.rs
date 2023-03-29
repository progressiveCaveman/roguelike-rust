use rltk;
use rltk::Point;
use shipyard::EntityId;
use crate::utils::point_diff;
use crate::{State};
use crate::gui::Palette;
use crate::{systems::system_particle::ParticleBuilder};
use crate::components::{Position, Monster, Viewshed, WantsToAttack, Confusion, WantsToPickupItem};
use crate::map::Map;

pub fn run_monster_ai_system(gs: &mut State) {

    let mut needs_wants_to_attack: Vec<EntityId> = Vec::new();
    let mut needs_wants_to_pick_up: Vec<(EntityId, EntityId)> = Vec::new();
    let mut to_update_confusion: Vec<(EntityId, Confusion)> = Vec::new();

    let mut to_try_move: Vec<(EntityId, Point)> = Vec::new();

    let world = &mut gs.world;
    let res = &mut gs.resources;

    {
        let map: &mut Map = &mut res.get_mut::<Map>().unwrap();
        let ppos: &Point = &res.get::<Point>().unwrap();
        let mut particle_builder = res.get_mut::<ParticleBuilder>().unwrap();


        // Monster ai
        for (id, (_mon, pos, vs)) in world.query::<(&Monster, &mut Position, &mut Viewshed)>().iter() {
            match world.get_mut::<Confusion>(id) {
                Err(_e) => {},
                Ok(confusion) => {
                    to_update_confusion.push((id, *confusion));
                    for pos in pos.ps.iter() {
                        particle_builder.request(pos.x, pos.y, 0.0, 0.0, Palette::COLOR_3, Palette::MAIN_BG, rltk::to_cp437('?'), 300.0);
                    }

                    // TODO attempt to move in a random direction

                    continue;
                }
            }

            // don't do anything if player is out of sight
            if !vs.visible_tiles.contains(&*ppos) {
                continue;
            }

            let mut retargeted = false;
            let mut target = *ppos;


            // TODO mutlitile monsters currently only attack from their first position
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(target, pos.ps[0]);
            if distance < 1.5 && !retargeted {
                needs_wants_to_attack.push(id);
            } else if vs.visible_tiles.contains(&target){

                // in order to stop multi-tile monsters from blocking themselves, make them not block before running A*
                // this is still just a hack since multi-tile monsters still path through 1 wide areas
                for pos in pos.ps.iter() {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = false;
                }
                let path = movement::get_path(map, pos.ps[0], target);

                // make monster block again
                for pos in pos.ps.iter() {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = true;
                }

                if path.success && path.steps.len() > 1 {
                    let p = map.idx_point(path.steps[1]);
                    to_try_move.push((id, point_diff(pos.ps[0], p)));
                }
            }
        }
    }

    for id in needs_wants_to_attack.iter() {
        let player_id: &EntityId = &res.get::<EntityId>().unwrap();
        world.add_component(*id, WantsToAttack {target: *player_id}).unwrap();
    }

    for (id, item) in needs_wants_to_pick_up.iter() {
        world.add_component(*id, WantsToPickupItem{ collected_by: *id, item: *item }).unwrap();
    }

    for (id, _confusion) in to_update_confusion.iter() {
        let mut to_remove = false;
        {
            let mut c = world.get_mut::<Confusion>(*id).unwrap();
            c.turns -= 1;
            if c.turns <= 0 { to_remove = true }
        }
        if to_remove { world.remove_one::<Confusion>(*id).unwrap(); }
    }

    for (id, delta) in to_try_move.iter() {
        try_move_entity(*id, *delta, gs);
    }
}
