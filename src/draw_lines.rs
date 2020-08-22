// experiment drawing lines with instancing. Transform is attribute params
// probably it's not faster

use glam::{vec2, Vec2, Vec3};
use miniquad::*;

use crate::camera::Camera;

#[rustfmt::skip]
pub const RECT: &[f32] = &[
    0., 0., 
    1., -0.75,
    1., 0.75,
    -1., 0.75,
    -1., -0.75,
];

#[rustfmt::skip]
const RECT_INDICES: &[u16] = &[
    0, 1, 2, 
    0, 2, 3,
    0, 3, 4,
    0, 4, 1
];
pub const MAX_RECT_NUM: usize = 100000;

#[derive(Debug, Default, Clone)]
pub struct Lines {
    pub positions: Vec<Vec2>,
    pub colors: Vec<Vec3>,
    pub scales: Vec<Vec2>,
    pub angles: Vec<f32>,
}

impl Lines {
    pub fn new_gpu_backed() -> Self {
        Lines {
            positions: Vec::with_capacity(MAX_RECT_NUM),
            colors: Vec::with_capacity(MAX_RECT_NUM),
            scales: Vec::with_capacity(MAX_RECT_NUM),
            angles: Vec::with_capacity(MAX_RECT_NUM),
        }
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.colors.clear();
        self.scales.clear();
        self.angles.clear();
    }

    pub fn add_raw(&mut self, pos: Vec2, scale: Vec2, angle: f32, color: Vec3,) {
        self.positions.push(pos);
        self.colors.push(color);
        self.scales.push(scale);
        self.angles.push(angle)
    }

    pub fn add(&mut self, from: Vec2, to: Vec2, thickness: f32, color: Vec3) {
        let dir = to - from;
        let length = dir.length();
        let angle = std::f32::consts::PI / 2. -  dir.y().atan2(dir.x());
        self.add_raw((from + to) / 2., vec2(thickness, length), angle, color)
    }

    pub fn extend(&mut self, segments: &Lines) {
        self.positions.extend(&segments.positions);
        self.colors.extend(&segments.colors);
        self.scales.extend(&segments.scales);
        self.angles.extend(&segments.angles);
    }
}

pub struct LinesRenderer {
    pipeline: Pipeline,
    bindings: Bindings,
    pub lines: Lines,
}

impl LinesRenderer {
    pub fn new(ctx: &mut Context) -> Self {
        let geometry_vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &RECT);
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &RECT_INDICES);
        let positions_vertex_buffer = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            MAX_RECT_NUM * std::mem::size_of::<Vec2>(),
        );

        let colors_vertex_buffer = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            MAX_RECT_NUM * std::mem::size_of::<Vec3>(),
        );

        let angles_vertex_buffer = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            MAX_RECT_NUM * std::mem::size_of::<f32>(),
        );

        let scales_vertex_buffer = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            MAX_RECT_NUM * std::mem::size_of::<Vec2>(),
        );

        let bindings = Bindings {
            vertex_buffers: vec![
                geometry_vertex_buffer,
                positions_vertex_buffer,
                colors_vertex_buffer,
                angles_vertex_buffer,
                scales_vertex_buffer,
            ],
            index_buffer,
            images: vec![],
        };

        let shader = Shader::new(
            ctx,
            hex_shader::VERTEX,
            hex_shader::FRAGMENT,
            hex_shader::META,
        );
        let pipeline = Pipeline::with_params(
            ctx,
            &[
                BufferLayout::default(),
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            // TODO refactor it in a single vertex. Because vertex assembly wouldn't be happy :)
            &[
                VertexAttribute::with_buffer("pos", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("inst_pos", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("color0", VertexFormat::Float3, 2),
                VertexAttribute::with_buffer("angle", VertexFormat::Float1, 3),
                VertexAttribute::with_buffer("scale", VertexFormat::Float2, 4),
            ],
            shader,
            PipelineParams {
                color_blend: Some((
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
        );
        LinesRenderer {
            lines: Lines::new_gpu_backed(),
            pipeline,
            bindings,
        }
    }

    pub fn clear_buffers(&mut self) {
        self.lines.clear();
    }

    /// WARNING: for GPU safety it's important that Lines are created with correct size (new_gpu_backed fn)
    pub fn push_segments(&mut self, ctx: &mut Context, lines: Lines) {
        self.lines.extend(&lines);
        self.bindings.vertex_buffers[1].update(ctx, &self.lines.positions[..]);
        self.bindings.vertex_buffers[2].update(ctx, &self.lines.colors[..]);
        self.bindings.vertex_buffers[3].update(ctx, &self.lines.angles[..]);
        self.bindings.vertex_buffers[4].update(ctx, &self.lines.scales[..]);
    }

    pub fn draw(&mut self, ctx: &mut Context, camera: &Camera) {
        let (width, height) = ctx.screen_size();
        let mvp = camera.get_mvp(height / width);

        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);
        ctx.apply_uniforms(&hex_shader::Uniforms { mvp });
        ctx.draw(
            0,
            RECT_INDICES.len() as i32,
            self.lines.positions.len() as i32,
        );
    }
}

pub fn _hex_mesh(r: f32) -> Vec<Vec2> {
    let step_angle = std::f32::consts::PI / 3.;
    (0..6)
        .map(|i| step_angle / 2. + step_angle * i as f32)
        .map(|a| vec2(f32::cos(a) * r, f32::sin(a) * r))
        .collect()
}

mod hex_shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec2 inst_pos;
    attribute vec3 color0;
    attribute float angle;
    attribute vec2 scale;
    
    varying lowp vec2 pp;
    varying lowp vec2 ip;
    varying lowp float a;
    varying lowp vec2 s;
    varying lowp vec4 color;

    uniform mat4 mvp;
    void main() {
        vec2 apos = 
            vec2(
                scale.x * pos.x * cos(angle) + scale.y * pos.y * sin(angle),
                -scale.x * pos.x * sin(angle) + scale.y * pos.y * cos(angle));
        vec4 new_pos = vec4(apos + inst_pos, 0.0, 1.0);
        lowp vec4 res_pos = mvp * new_pos;
        gl_Position = res_pos;
        
        pp = vec2(new_pos.x, new_pos.y);
        ip = inst_pos;
        a = angle;
        s = scale;
        color = vec4(color0, 0.5);
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 pp;
    varying lowp vec2 ip;
    varying lowp float a;
    varying lowp vec2 s;
    varying lowp vec4 color;
    uniform highp mat4 mvp;
    const lowp float aaborder = 0.0045;

    lowp float line_segment(in lowp vec2 p, in lowp vec2 a, in lowp vec2 b) {
        lowp vec2 ba = b - a;
        lowp vec2 pa = p - a;
        lowp float h = clamp(dot(pa, ba) / dot(ba, ba), 0., 1.);
        return length(pa - h * ba);
    }

    void main() {
        lowp mat2 rot = mat2(cos(a), -sin(a),
                        sin(a), cos(a));
        lowp vec2 a = ip + rot * vec2(0.0, -s.y / 2.);
        lowp vec2 b = ip + rot * vec2(0.0, s.y / 2.);
        lowp float d = line_segment(pp, a, b) - s.x ;
        lowp float scaled_border = min(aaborder / mvp[0][0], 0.1);
        lowp float edge1 = -scaled_border;
        lowp float edge2 = 0.;

        if (d < 0.) {
            lowp float smooth = 1.;
            if (d > edge1) {
                smooth = 1. - smoothstep(edge1, edge2, d);
            }
            lowp vec4 color = color;
            color.a = smooth;
            gl_FragColor = color;
        } else {
            gl_FragColor = vec4(color.xyz, 0.0);
        }
    }
    "#;

    pub const META: ShaderMeta = ShaderMeta {
        images: &[],
        uniforms: UniformBlockLayout {
            uniforms: &[("mvp", UniformType::Mat4)],
        },
    };

    #[repr(C)]
    pub struct Uniforms {
        pub mvp: glam::Mat4,
    }
}
