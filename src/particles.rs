use macroquad::prelude::*;

use crate::assets::Assets;

pub trait Particle {
    fn draw(&mut self, time: f32, assets: &Assets);
}

pub trait ScreenParticle {
    fn draw(&mut self, time: f32, assets: &Assets, scale_factor: f32, offset: Vec2);
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
pub struct TextParticle {
    pub text: String,
    pub color: Color,
    pub origin: Vec2,
}
impl ScreenParticle for TextParticle {
    fn draw(&mut self, time: f32, assets: &Assets, scale_factor: f32, offset: Vec2) {
        let modulated = 2.0 - (2.5 * time - 1.0).powi(2);
        let screen = (self.origin + vec2(0.0, -modulated * 1.0)) * scale_factor + offset;
        draw_text_ex(
            &self.text,
            screen.x,
            screen.y + 6.0 * scale_factor,
            TextParams {
                color: self.color,
                font: Some(&assets.font),
                font_size: (scale_factor * 6.0) as u16,
                ..Default::default()
            },
        );
    }
}
