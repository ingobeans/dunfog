use macroquad::{miniquad::window::screen_size, prelude::*};
use utils::*;

mod assets;
mod utils;

#[macroquad::main("dunfog")]
async fn main() {
    let assets = assets::Assets::default();
    let camera = create_camera(SCREEN_WIDTH, SCREEN_HEIGHT);
    loop {
        let (actual_screen_width, actual_screen_height) = screen_size();
        let scale_factor =
            (actual_screen_width / SCREEN_WIDTH).min(actual_screen_height / SCREEN_HEIGHT);
        set_camera(&camera);
        clear_background(WHITE);
        assets.tileset.draw_sprite(0.0, 0.0, 0.0, 0.0, None);

        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &camera.render_target.as_ref().unwrap().texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(
                    SCREEN_WIDTH * scale_factor,
                    SCREEN_HEIGHT * scale_factor,
                )),
                ..Default::default()
            },
        );
        next_frame().await
    }
}
