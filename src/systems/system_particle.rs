use resources::Resources;
use rltk::{FontCharType, Rltk, Point, RGBA};

use crate::{RenderOrder, components::{Lifetime, Particle, Position, Renderable, Velocity}};

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

pub fn spawn_particles(world: &mut World, res: &mut Resources) {
    let mut particle_builder = res.get_mut::<ParticleBuilder>().unwrap();
    for p in particle_builder.requests.iter() {
        let _id = world.spawn((
            Renderable {glyph: p.glyph, fg: p.fg, bg: p.bg, always_render: true, order: RenderOrder::Particle, ..Default::default()},
            Position {ps: vec![Point{x: p.x, y: p.y}]},
            Velocity {x: p.vel_x, y: p.vel_y},
            Lifetime {ms: p.lifetime_ms},
            Particle {float_x: p.x as f32, float_y: p.y as f32}
        ));
    }
    particle_builder.clear();
}

pub fn update_particles(world: &mut World, _res: &mut Resources, ctx: &Rltk) {
    for (id, (particle, lifetime)) in world.query::<(&mut Particle, &mut Lifetime)>().iter() {
        lifetime.ms -= ctx.frame_time_ms;

        let vel = world.get::<Velocity>(id);
        if let Ok(vel) = vel {
            for pos in world.get_mut::<Position>(id).unwrap().ps.iter_mut() {
                particle.float_x += (vel.x) * (ctx.frame_time_ms / 1000.0);
                particle.float_y += (vel.y) * (ctx.frame_time_ms / 1000.0);
                pos.x = particle.float_x as i32;
                pos.y = particle.float_y as i32;
            }
        }
    }

    remove_dead_particles(world);
}

pub fn remove_dead_particles(world: &mut World) {
    let mut particles_to_remove: Vec<EntityId> = Vec::new();
    for (id, lifetime) in world.query::<&mut Lifetime>().iter() {
        if lifetime.ms <= 0.0 {
            particles_to_remove.push(id);
        }
    }

    for id in particles_to_remove {
        world.despawn(id).unwrap();
    }
}
