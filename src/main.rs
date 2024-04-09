use std::f32::consts::PI;

use macroquad::prelude::*;

pub mod animation;
use animation::Animation;

#[macroquad::main("Template")]
async fn main() {
    let mut animation = Animation::new(640.0, 360.0, None);
    animation.enable_fxaa();

    let mut rot = 0.0;
    loop {
        animation.set_camera();
        draw_line(-30.0, -100.0, 30.0, -100.0, 3.0, BLUE);
        rot += 0.01;
        draw_circle(-100.0, 35.0, 20.0, YELLOW);
        draw_circle(100.0, 35.0, 20.0, GREEN);
        draw_triangle(vec2(-70., 0.0), vec2(0., 70.), vec2(70., 0.), BLUE);
        draw_rectangle_ex(
            0.0,
            0.0,
            30.0,
            100.0,
            DrawRectangleParams {
                rotation: PI / 2. + rot,
                offset: vec2(0.5, 0.5),
                ..Default::default()
            },
        );

        animation.set_default_camera();
        animation.draw_frame();

        next_frame().await;
    }
}
