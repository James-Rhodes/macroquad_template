use macroquad::{miniquad::window::screen_size, prelude::*};
enum RenderState {
    CameraRendering,
    ScreenRendering,
}

pub struct Animation {
    render_target: RenderTarget,
    camera: Camera2D,
    pub bg_color: Color,
    render_state: RenderState,
    pub draw_size: Vec2,
    material: Option<Material>,
}

impl Animation {
    pub fn new(width: f32, height: f32, bg_color: Option<Color>) -> Self {
        // Screen dimensions will be:
        //     x: -width/2 -> width/2 (left -> right)
        //     y: -height/2 -> height/2 (bottom -> top)
        let render_target = render_target(width as u32, height as u32);
        render_target.texture.set_filter(FilterMode::Linear);

        let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., width, height));

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
            draw_size: vec2(width, height),
            material: None,
        }
    }

    pub fn get_world_mouse(&self) -> Vec2 {
        let mouse: Vec2 = mouse_position().into();
        self.screen_to_world(mouse)
    }

    pub fn screen_to_world(&self, mut point: Vec2) -> Vec2 {
        // Move window space to -1 -> 1
        point.x = (point.x - screen_width() / 2.0) / (screen_width() * 0.5);
        point.y = -(point.y - screen_height() / 2.0) / (screen_height() * 0.5);

        let screen_size: Vec2 = screen_size().into();
        let mut left_over_space: Vec2 = screen_size - self.draw_size; // in pixels
        left_over_space /= screen_size; // as a percentage 0 -> 1
        left_over_space += vec2(1., 1.); // We want the position to be 100% plus any left over
                                         // space

        // Convert from -1 -> 1 to render size + what ever percent is left over (so the borders
        // don't mess up the conversions)
        point.y *= 0.5 * left_over_space.y * self.render_target.texture.height();
        point.x *= 0.5 * left_over_space.x * self.render_target.texture.width();
        point
    }

    pub fn enable_fxaa(&mut self) {
        let uniforms = vec![("texture_size".to_string(), UniformType::Float2)];
        let material = load_material(
            ShaderSource::Glsl {
                vertex: FXAA_VERTEX_SHADER,
                fragment: FXAA_FRAGMENT_SHADER,
            },
            MaterialParams {
                uniforms,
                ..Default::default()
            },
        )
        .unwrap();

        self.material = Some(material);
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

    pub fn draw_frame(&mut self) {
        if matches!(self.render_state, RenderState::CameraRendering) {
            panic!("ResizeableScreen::set_default_camera must be called before you can draw the frame to the screen");
        }

        if let Some(material) = &self.material {
            material.set_uniform("texture_size", self.draw_size);
            gl_use_material(material);
        } else {
            gl_use_default_material();
        }

        // TODO: Uncomment this
        // clear_background(self.bg_color);

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

        self.draw_size = vec2(draw_width, draw_height);

        draw_texture_ex(
            &self.render_target.texture,
            new_x,
            new_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(self.draw_size),
                ..Default::default()
            },
        );
    }
}

const FXAA_VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;

varying lowp vec2 uv;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;
const FXAA_FRAGMENT_SHADER: &str = r#"
#version 100
precision lowp float;

// THIS CODE IS THANKS TO: 
// https://blog.simonrodriguez.fr/articles/2016/07/implementing_fxaa.html

varying vec2 uv;

// UNIFORMS
uniform sampler2D Texture;
uniform vec2 texture_size;

// CONSTANTS
const float EDGE_THRESHOLD_MIN = 0.0312;
const float EDGE_THRESHOLD_MAX = 0.125;
const int ITERATIONS = 12;
const float SUBPIXEL_QUALITY = 0.75;

float rgb2luma(vec3 rgb){
    return sqrt(dot(rgb, vec3(0.299, 0.587, 0.114)));
}

float get_quality(int i) {
    float QUALITY[12];
    QUALITY[0] = 1.0;
    QUALITY[1] = 1.0;
    QUALITY[2] = 1.0;
    QUALITY[3] = 1.0;
    QUALITY[4] = 1.0;
    QUALITY[5] = 1.0;
    QUALITY[6] = 1.5;
    QUALITY[7] = 2.0;
    QUALITY[8] = 2.0;
    QUALITY[9] = 2.0;
    QUALITY[10] = 4.0;
    QUALITY[11] = 8.0;
    return QUALITY[i];
}

void main() {
    vec2 inverseScreenSize = vec2(1.0/texture_size.x, 1.0/texture_size.y);
    vec3 colorCenter = texture2D(Texture,uv).rgb;

    // Luma at the current fragment
    float lumaCenter = rgb2luma(colorCenter);

    // Directions
    vec2 up = vec2(0., -inverseScreenSize.y);
    vec2 down = vec2(0., inverseScreenSize.y);
    vec2 left = vec2(-inverseScreenSize.x, 0.);
    vec2 right = vec2(inverseScreenSize.x, 0.);

    // Luma at the four direct neighbours of the current fragment.
    float lumaDown = rgb2luma(texture2D(Texture, uv + down).rgb);
    float lumaUp = rgb2luma(texture2D(Texture,uv + up).rgb);
    float lumaLeft = rgb2luma(texture2D(Texture,uv + left).rgb);
    float lumaRight = rgb2luma(texture2D(Texture,uv + right).rgb);

    // Find the maximum and minimum luma around the current fragment.
    float lumaMin = min(lumaCenter,min(min(lumaDown,lumaUp),min(lumaLeft,lumaRight)));
    float lumaMax = max(lumaCenter,max(max(lumaDown,lumaUp),max(lumaLeft,lumaRight)));

    // Compute the delta.
    float lumaRange = lumaMax - lumaMin;

    // If the luma variation is lower that a threshold (or if we are in a really dark area), we are not on an edge, don't perform any AA.
    if(lumaRange < max(EDGE_THRESHOLD_MIN,lumaMax*EDGE_THRESHOLD_MAX)){
        gl_FragColor = vec4(colorCenter, 1.0);
        return;
    }

    // Query the 4 remaining corners lumas.
    float lumaDownLeft = rgb2luma(texture2D(Texture,uv + down + left).rgb);
    float lumaUpRight = rgb2luma(texture2D(Texture,uv + up + right).rgb);
    float lumaUpLeft = rgb2luma(texture2D(Texture,uv + up + left).rgb);
    float lumaDownRight = rgb2luma(texture2D(Texture,uv + down + right).rgb);

    // Combine the four edges lumas (using intermediary variables for future computations with the same values).
    float lumaDownUp = lumaDown + lumaUp;
    float lumaLeftRight = lumaLeft + lumaRight;

    // Same for corners
    float lumaLeftCorners = lumaDownLeft + lumaUpLeft;
    float lumaDownCorners = lumaDownLeft + lumaDownRight;
    float lumaRightCorners = lumaDownRight + lumaUpRight;
    float lumaUpCorners = lumaUpRight + lumaUpLeft;

    // Compute an estimation of the gradient along the horizontal and vertical axis.
    float edgeHorizontal =  abs(-2.0 * lumaLeft + lumaLeftCorners)  + abs(-2.0 * lumaCenter + lumaDownUp ) * 2.0    + abs(-2.0 * lumaRight + lumaRightCorners);
    float edgeVertical =    abs(-2.0 * lumaUp + lumaUpCorners)      + abs(-2.0 * lumaCenter + lumaLeftRight) * 2.0  + abs(-2.0 * lumaDown + lumaDownCorners);

    // Is the local edge horizontal or vertical ?
    bool isHorizontal = (edgeHorizontal >= edgeVertical);

    // Select the two neighboring texels lumas in the opposite direction to the local edge.
    float luma1 = isHorizontal ? lumaDown : lumaLeft;
    float luma2 = isHorizontal ? lumaUp : lumaRight;
    // Compute gradients in this direction.
    float gradient1 = luma1 - lumaCenter;
    float gradient2 = luma2 - lumaCenter;

    // Which direction is the steepest ?
    bool is1Steepest = abs(gradient1) >= abs(gradient2);

    // Gradient in the corresponding direction, normalized.
    float gradientScaled = 0.25*max(abs(gradient1),abs(gradient2));

    // Choose the step size (one pixel) according to the edge direction.
    float stepLength = isHorizontal ? inverseScreenSize.y : inverseScreenSize.x;

    // Average luma in the correct direction.
    float lumaLocalAverage = 0.0;

    if(is1Steepest){
        // Switch the direction
        stepLength = - stepLength;
        lumaLocalAverage = 0.5*(luma1 + lumaCenter);
    } else {
        lumaLocalAverage = 0.5*(luma2 + lumaCenter);
    }

    // Shift UV in the correct direction by half a pixel.
    vec2 currentUv = uv;
    if(isHorizontal){
        currentUv.y += stepLength * 0.5;
    } else {
        currentUv.x += stepLength * 0.5;
    }

    // Compute offset (for each iteration step) in the right direction.
    vec2 offset = isHorizontal ? vec2(inverseScreenSize.x,0.0) : vec2(0.0,inverseScreenSize.y);
    // Compute UVs to explore on each side of the edge, orthogonally. The QUALITY allows us to step faster.
    vec2 uv1 = currentUv - offset;
    vec2 uv2 = currentUv + offset;

    // Read the lumas at both current extremities of the exploration segment, and compute the delta wrt to the local average luma.
    float lumaEnd1 = rgb2luma(texture2D(Texture,uv1).rgb);
    float lumaEnd2 = rgb2luma(texture2D(Texture,uv2).rgb);
    lumaEnd1 -= lumaLocalAverage;
    lumaEnd2 -= lumaLocalAverage;

    // If the luma deltas at the current extremities are larger than the local gradient, we have reached the side of the edge.
    bool reached1 = abs(lumaEnd1) >= gradientScaled;
    bool reached2 = abs(lumaEnd2) >= gradientScaled;
    bool reachedBoth = reached1 && reached2;

    // If the side is not reached, we continue to explore in this direction.
    if(!reached1){
        uv1 -= offset;
    }
    if(!reached2){
        uv2 += offset;
    }

    // If both sides have not been reached, continue to explore.
    if(!reachedBoth){

        for(int i = 2; i < ITERATIONS; i++){
            // If needed, read luma in 1st direction, compute delta.
            if(!reached1){
                lumaEnd1 = rgb2luma(texture2D(Texture, uv1).rgb);
                lumaEnd1 = lumaEnd1 - lumaLocalAverage;
            }
            // If needed, read luma in opposite direction, compute delta.
            if(!reached2){
                lumaEnd2 = rgb2luma(texture2D(Texture, uv2).rgb);
                lumaEnd2 = lumaEnd2 - lumaLocalAverage;
            }
            // If the luma deltas at the current extremities is larger than the local gradient, we have reached the side of the edge.
            reached1 = abs(lumaEnd1) >= gradientScaled;
            reached2 = abs(lumaEnd2) >= gradientScaled;
            reachedBoth = reached1 && reached2;

            // If the side is not reached, we continue to explore in this direction, with a variable quality.
            if(!reached1){
                uv1 -= offset * get_quality(i);
            }
            if(!reached2){
                uv2 += offset * get_quality(i);
            }

            // If both sides have been reached, stop the exploration.
            if(reachedBoth){ break;}
        }
    }

    // Compute the distances to each extremity of the edge.
    float distance1 = isHorizontal ? (uv.x - uv1.x) : (uv.y - uv1.y);
    float distance2 = isHorizontal ? (uv2.x - uv.x) : (uv2.y - uv.y);

    // which direction is the extremity of the edge closer ?
    bool isDirection1 = distance1 < distance2;
    float distanceFinal = min(distance1, distance2);

    // Length of the edge.
    float edgeThickness = (distance1 + distance2);

    // UV offset: read in the direction of the closest side of the edge.
    float pixelOffset = - distanceFinal / edgeThickness + 0.5;

    // Is the luma at center smaller than the local average ?
    bool isLumaCenterSmaller = lumaCenter < lumaLocalAverage;

    // If the luma at center is smaller than at its neighbour, the delta luma at each end should be positive (same variation).
    // (in the direction of the closer side of the edge.)
    bool correctVariation = ((isDirection1 ? lumaEnd1 : lumaEnd2) < 0.0) != isLumaCenterSmaller;

    // If the luma variation is incorrect, do not offset.
    float finalOffset = correctVariation ? pixelOffset : 0.0;

    // Sub-pixel shifting
    // Full weighted average of the luma over the 3x3 neighborhood.
    float lumaAverage = (1.0/12.0) * (2.0 * (lumaDownUp + lumaLeftRight) + lumaLeftCorners + lumaRightCorners);
    // Ratio of the delta between the global average and the center luma, over the luma range in the 3x3 neighborhood.
    float subPixelOffset1 = clamp(abs(lumaAverage - lumaCenter)/lumaRange,0.0,1.0);
    float subPixelOffset2 = (-2.0 * subPixelOffset1 + 3.0) * subPixelOffset1 * subPixelOffset1;
    // Compute a sub-pixel offset based on this delta.
    float subPixelOffsetFinal = subPixelOffset2 * subPixelOffset2 * SUBPIXEL_QUALITY;

    // Pick the biggest of the two offsets.
    finalOffset = max(finalOffset,subPixelOffsetFinal);

    // Compute the final UV coordinates.
    vec2 finalUv = uv;
    if(isHorizontal){
        finalUv.y += finalOffset * stepLength;
    } else {
        finalUv.x += finalOffset * stepLength;
    }

    // Read the color at the new UV coordinates, and use it.
    vec3 finalColor = texture2D(Texture,finalUv).rgb;

    gl_FragColor = vec4(finalColor, 1.0);
}"#;
