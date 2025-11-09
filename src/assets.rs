use asefile::{self, AsepriteFile};
use image::EncodableLayout;
use macroquad::prelude::*;

pub struct Assets {
    pub tileset: Spritesheet,
    pub items: Spritesheet,
    pub inventory: Texture2D,
    pub darkness: Texture2D,
    pub semi_darkness: Texture2D,
    pub hp: Texture2D,
    pub hover_card: Texture2D,
    pub font: Font,
    pub ctx_menu: Texture2D,
}
impl Default for Assets {
    fn default() -> Self {
        Self {
            tileset: Spritesheet::new(
                load_ase_texture(include_bytes!("../assets/spritesheet.ase"), None),
                8.0,
            ),
            items: Spritesheet::new(
                load_ase_texture(include_bytes!("../assets/items.ase"), None),
                8.0,
            ),
            inventory: load_ase_texture(include_bytes!("../assets/inventory.ase"), None),
            darkness: load_ase_texture(include_bytes!("../assets/darkness.ase"), None),
            semi_darkness: load_ase_texture(include_bytes!("../assets/semi_darkness.ase"), None),
            hp: load_ase_texture(include_bytes!("../assets/hp.ase"), None),
            hover_card: load_ase_texture(include_bytes!("../assets/hover_card.ase"), None),
            ctx_menu: load_ase_texture(include_bytes!("../assets/ctx_menu.ase"), None),
            font: load_ttf_font_from_bytes(include_bytes!("../assets/novem.ttf")).unwrap(),
        }
    }
}

fn load_ase_texture(bytes: &[u8], layer: Option<u32>) -> Texture2D {
    let img = AsepriteFile::read(bytes).unwrap();
    let img = if let Some(layer) = layer {
        img.layer(layer).frame(0).image()
    } else {
        img.frame(0).image()
    };
    let new = Image {
        width: img.width() as u16,
        height: img.height() as u16,
        bytes: img.as_bytes().to_vec(),
    };
    let texture = Texture2D::from_image(&new);
    texture.set_filter(FilterMode::Nearest);
    texture
}
pub struct Spritesheet {
    pub texture: Texture2D,
    pub sprite_size: f32,
}
impl Spritesheet {
    pub fn new(texture: Texture2D, sprite_size: f32) -> Self {
        Self {
            texture,
            sprite_size,
        }
    }
    #[expect(dead_code)]
    /// Same as `draw_tile`, except centered
    pub fn draw_sprite(
        &self,
        screen_x: f32,
        screen_y: f32,
        tile_x: f32,
        tile_y: f32,
        params: Option<&DrawTextureParams>,
    ) {
        self.draw_tile(
            screen_x - self.sprite_size / 2.0,
            screen_y - self.sprite_size / 2.0,
            tile_x,
            tile_y,
            params,
        );
    }
    /// Draws a single tile from the spritesheet
    pub fn draw_tile(
        &self,
        screen_x: f32,
        screen_y: f32,
        tile_x: f32,
        tile_y: f32,
        params: Option<&DrawTextureParams>,
    ) {
        let mut p = params.cloned().unwrap_or(DrawTextureParams::default());
        p.dest_size = p
            .dest_size
            .or(Some(Vec2::new(self.sprite_size, self.sprite_size)));
        p.source = p.source.or(Some(Rect {
            x: tile_x * self.sprite_size,
            y: tile_y * self.sprite_size,
            w: self.sprite_size,
            h: self.sprite_size,
        }));
        draw_texture_ex(&self.texture, screen_x, screen_y, WHITE, p);
    }
}
