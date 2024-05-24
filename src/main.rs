use std::f32::consts::PI;

use macroquad::prelude::*;
use mqanim::Animation;

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 360.0;
fn window_conf() -> Conf {
    Conf {
        window_title: "Template".to_owned(),
        sample_count: 4,
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut animation = Animation::new(WINDOW_WIDTH, WINDOW_HEIGHT, None);
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
            100.0,
            200.0,
            DrawRectangleParams {
                rotation: PI / 2. + rot,
                offset: vec2(0.5, 0.5),
                ..Default::default()
            },
        );
        let mouse = animation.get_world_mouse();
        draw_circle(mouse.x, mouse.y, 10.0, ORANGE);

        animation.set_default_camera();
        animation.draw_frame();

        next_frame().await;
    }
}
