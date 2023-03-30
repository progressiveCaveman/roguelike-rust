use rltk::{FontCharType, Point, RGBA};
use shipyard::{EntityId, Unique, ViewMut, View, IntoIter, IntoWithId, Get, AllStoragesViewMut, UniqueView, UniqueViewMut};

use crate::{RenderOrder, components::{Lifetime, Particle, Position, Renderable, Velocity}, utils::FrameTime};

#[derive(Debug)]
struct ParticleRequest {
    x: i32,
    y: i32,
    vel_x: f32,
    vel_y: f32,
    fg: RGBA,
    bg: RGBA,
    glyph: FontCharType,
    lifetime_ms: f32
}

#[derive(Debug, Unique)]
pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>
}

impl ParticleBuilder {
    pub fn new() -> ParticleBuilder {
        ParticleBuilder{ requests: Vec::new() }
    }

    pub fn request(&mut self, x: i32, y: i32, vel_x: f32, vel_y: f32, fg: RGBA, bg: RGBA, glyph: FontCharType, lifetime_ms: f32) {
        self.requests.push(
            ParticleRequest {
                x, y, vel_x, vel_y, fg, bg, glyph, lifetime_ms
            }
        );
    }

    pub fn clear(&mut self) {
        self.requests.clear();
    }
}

pub fn spawn_particles(mut particle_builder: UniqueViewMut<ParticleBuilder>, mut store: AllStoragesViewMut) {
    for p in particle_builder.requests.iter() {
        let _id = store.add_entity((
            Renderable {glyph: p.glyph, fg: p.fg, bg: p.bg, always_render: true, order: RenderOrder::Particle, ..Default::default()},
            Position {ps: vec![Point{x: p.x, y: p.y}]},
            Velocity {x: p.vel_x, y: p.vel_y},
            Lifetime {ms: p.lifetime_ms},
            Particle {float_x: p.x as f32, float_y: p.y as f32}
        ));
    }
    particle_builder.clear();
}

pub fn update_particles(store: AllStoragesViewMut, frametime: UniqueView<FrameTime>, mut vpart: ViewMut<Particle>, mut vlifetime: ViewMut<Lifetime>, vvel: View<Velocity>, vpos: ViewMut<Position>) {
    for (id, (particle, lifetime)) in (&mut vpart, &mut vlifetime).iter().with_id() {
        lifetime.ms -= frametime.0;

        let vel = vvel.get(id);
        if let Ok(vel) = vel {
            for pos in vpos.get(id).unwrap().ps.iter_mut() {
                particle.float_x += (vel.x) * (frametime.0 / 1000.0);
                particle.float_y += (vel.y) * (frametime.0 / 1000.0);
                pos.x = particle.float_x as i32;
                pos.y = particle.float_y as i32;
            }
        }
    }

    remove_dead_particles(store, vlifetime);
}

pub fn remove_dead_particles(mut store: AllStoragesViewMut, vlifetime: ViewMut<Lifetime>) {
    let mut particles_to_remove: Vec<EntityId> = Vec::new();
    for (id, lifetime) in vlifetime.iter().with_id() {//world.query::<&mut Lifetime>().iter() {
        if lifetime.ms <= 0.0 {
            particles_to_remove.push(id);
        }
    }

    for id in particles_to_remove {
        store.delete_entity(id);
        // world.despawn(id).unwrap();
    }
}
