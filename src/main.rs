use miniquad::*;
mod camera;
mod draw_lines;

use crate::camera::Camera;
use draw_lines::*;
use glam::{vec2, vec3, Vec2};
use quad_rand as qrand;
pub const MAP_SIZE: i32 = 11;

pub struct Mouse {
    pub left_down: bool,
    pub last_left_down: Vec2,
    pub position: Vec2,
    pub position_screen: Vec2,
}

impl Mouse {
    pub fn update(&mut self, pos: Vec2, camera: &Camera, width: f32, height: f32) {
        self.position_screen = pos;
        self.position = camera.unproject(pos.x(), pos.y(), width, height);
    }
}

impl Default for Mouse {
    fn default() -> Self {
        Mouse {
            left_down: false,
            last_left_down: vec2(0., 0.),
            position: vec2(0., 0.),
            position_screen: vec2(0., 0.),
        }
    }
}

struct Stage {
    mouse: Mouse,
    lines_renderer: LinesRenderer,
    camera: Camera,
    lines: Lines,
}

impl Stage {
    pub fn new(ctx: &mut Context) -> Stage {
        let mut lines = Lines::new_gpu_backed();
        let mut prev = vec2(0., 0.);
        for _ in 0..10_000 {
            let  point = vec2(qrand::gen_range(-100., 100.), qrand::gen_range(-100., 100.));
            lines.add(
                prev, 
                point, 
                0.1, 
                vec3(qrand::gen_range(0.5, 1.), qrand::gen_range(0., 1.), qrand::gen_range(0., 1.))
            );
            prev = point;
        }

        Stage {
            lines_renderer: LinesRenderer::new(ctx),
            camera: Camera::default(),
            lines,
            mouse: Mouse::default(),
        }
    }
}

impl EventHandler for Stage {
    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        match button {
            MouseButton::Left => {
                self.mouse.last_left_down = vec2(x, y);
                self.mouse.left_down = true;
            }
            MouseButton::Right => {}
            _ => (),
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        if MouseButton::Left == button {
            self.mouse.left_down = false;
        }
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        let (width, height) = ctx.screen_size();
        if self.mouse.left_down {
            let pos = vec2(x, y);
            let mut delta = (pos - self.mouse.last_left_down) / self.camera.zoom;
            delta.set_x(-1. * delta.x());
            delta.set_x(delta.x() / width);
            delta.set_y(delta.y() / height / 2.); // why /2. ??
            self.camera.position_add(delta, MAP_SIZE as f32);
            self.mouse.last_left_down = pos;
        }
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        let y = y.signum() * 1.2;
        self.camera.zoom_wheel(y);
        self.camera.update();
    }

    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        ctx.begin_default_pass(PassAction::clear_color(1., 0.98, 200. / 255., 1.));
        self.lines_renderer.clear_buffers();
        self.lines_renderer.push_segments(ctx, self.lines.clone());
        self.lines_renderer.draw(ctx, &self.camera);
        ctx.end_render_pass();
        ctx.commit_frame();
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), |mut ctx| {
        UserData::owning(Stage::new(&mut ctx), ctx)
    });
}
