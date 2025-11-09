use macroquad::prelude::*;

use crate::assets::Assets;

pub trait Particle {
    fn draw(&mut self, time: f32, assets: &Assets);
}

pub struct ProjectileParticle {
    pub sprite_x: f32,
    pub sprite_y: f32,
    pub origin: Vec2,
    pub dest: Vec2,
}
impl Particle for ProjectileParticle {
    fn draw(&mut self, time: f32, assets: &Assets) {
        let screen = self.origin.lerp(self.dest, time);
        assets.particles.draw_sprite(
            screen.x,
            screen.y,
            self.sprite_x,
            self.sprite_y,
            Some(&DrawTextureParams {
                rotation: (self.dest - self.origin).to_angle(),
                ..Default::default()
            }),
        );
    }
}
