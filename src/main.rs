use std::f32::consts::PI;

use macroquad::{miniquad::window::screen_size, prelude::*};

enum RenderState {
    CameraRendering,
    ScreenRendering,
}

struct Animation {
    render_target: RenderTarget,
    camera: Camera2D,
    pub bg_color: Color,
    render_state: RenderState,
}

impl Animation {
    pub fn new(width: f32, height: f32, bg_color: Option<Color>) -> Self {
        // Screen dimensions will be:
        //     x: -width/2 -> width/2 (left -> right)
        //     y: -height/2 -> height/2 (bottom -> top)
        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., width, height));
        let render_target = render_target(width as u32, height as u32);
        render_target.texture.set_filter(FilterMode::Linear);

        camera.render_target = Some(render_target.clone());
        camera.target = vec2(0., 0.);

        let bg_color = if let Some(bg_color) = bg_color {
            bg_color
        } else {
            Color {
                r: 43. / 255.,
                g: 44. / 255.,
                b: 47. / 255.,
                a: 0.,
            }
        };

        Self {
            render_target,
            camera,
            bg_color,
            render_state: RenderState::ScreenRendering,
        }
    }

    pub fn set_camera(&mut self) {
        set_camera(&self.camera);
        clear_background(self.bg_color);
        self.render_state = RenderState::CameraRendering;
    }

    pub fn set_default_camera(&mut self) {
        self.render_state = RenderState::ScreenRendering;
        set_default_camera();
    }

    pub fn draw_frame(&self) {
        // TODO: Anti aliasing better probably in here with a custom shader
        if matches!(self.render_state, RenderState::CameraRendering) {
            panic!("ResizeableScreen::set_default_camera must be called before you can draw the frame to the screen");
        }

        clear_background(self.bg_color);

        let (sw, sh) = screen_size();

        let tex_size = self.render_target.texture.size();
        let tw = tex_size.x;
        let th = tex_size.y;

        let h_scale = sw / tw;
        let v_scale = sh / th;

        let scale = h_scale.min(v_scale);

        let draw_width = tw * scale;
        let draw_height = th * scale;
        let mut new_x: f32 = 0.0;
        let mut new_y: f32 = 0.0;

        // Set the location to be in the center of the screen always
        if h_scale < v_scale {
            let left_over_space = sh - draw_height;
            new_y = left_over_space / 2.0;
        } else {
            let left_over_space = sw - draw_width;
            new_x = left_over_space / 2.0;
        }

        draw_texture_ex(
            &self.render_target.texture,
            new_x,
            new_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(draw_width, draw_height)),
                ..Default::default()
            },
        );
    }
}

#[macroquad::main("Template")]
async fn main() {
    let mut animation = Animation::new(320.0, 150.0, None);

    let mut rot = 0.0;
    loop {
        animation.set_camera();
        draw_line(-30.0, -45.0, 30.0, -45.0, 3.0, BLUE);
        rot += 0.01;
        draw_circle(-45.0, 35.0, 20.0, YELLOW);
        draw_circle(45.0, 35.0, 20.0, GREEN);
        draw_triangle(vec2(-50., 0.0), vec2(0., 50.), vec2(50., 0.), BLUE);
        draw_rectangle_ex(
            0.0,
            0.0,
            30.0,
            45.0,
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
