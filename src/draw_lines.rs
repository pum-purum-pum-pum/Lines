// experiment drawing lines with instancing. Transform is attribute params
// probably it's not faster

use glam::{Vec2, Vec3};
use miniquad::*;

use crate::camera::Camera;

#[rustfmt::skip]
pub const RECT: &[f32] = &[
    0., 0., 
    1.1, -1.,
    1.1, 1.,
    -1.1, 1.,
    -1.1, -1.,
];

#[rustfmt::skip]
const RECT_INDICES: &[u16] = &[
    0, 1, 2, 
    0, 2, 3,
    0, 3, 4,
    0, 4, 1
];

#[repr(u8)]
pub enum SegmentType {
    All = 0,
    NoFirst = 1,
    NoSecond = 2,
    NoAll = 3,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Line {
    pub segment_type: f32,
    pub position: Vec2,
    pub thickness: f32,
    pub dir: Vec2,
    pub color: Vec3,
}

impl Line {
    pub fn new(
        segment_type: SegmentType,
        from: Vec2,
        to: Vec2,
        thickness: f32,
        color: Vec3,
    ) -> Self {
        let dir = to - from;
        Line {
            segment_type: segment_type as u8 as f32,
            position: (from + to) / 2.,
            thickness,
            dir,
            color,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Lines(Vec<Line>);

impl Lines {
    fn new_gpu_backed(max_lines_num: usize) -> Self {
        Lines(Vec::with_capacity(max_lines_num))
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn add(&mut self, line: Line) {
        self.0.push(line);
    }

    pub fn extend(&mut self, segments: &Lines) {
        self.0.extend(segments.0.iter())
    }
}

pub struct LinesRenderer {
    pipeline: Pipeline,
    bindings: Bindings,
    pub lines: Lines,
    max_lines_num: usize,
}

impl LinesRenderer {
    pub fn new(ctx: &mut Context, max_lines_num: usize) -> Self {
        let geometry_vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &RECT);
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &RECT_INDICES);
        let lines_vertex_buffer = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            max_lines_num * std::mem::size_of::<Line>(),
        );
        let bindings = Bindings {
            vertex_buffers: vec![geometry_vertex_buffer, lines_vertex_buffer],
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
            ],
            &[
                VertexAttribute::with_buffer("pos", VertexFormat::Float2, 0),
                VertexAttribute::with_buffer("segment_type", VertexFormat::Float1, 1),
                VertexAttribute::with_buffer("inst_pos", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("thickness", VertexFormat::Float1, 1),
                VertexAttribute::with_buffer("dir", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("color0", VertexFormat::Float3, 1),
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
            lines: Lines::new_gpu_backed(max_lines_num),
            pipeline,
            bindings,
            max_lines_num,
        }
    }

    pub fn create_lines(&self) -> Lines {
        Lines::new_gpu_backed(self.max_lines_num)
    }

    pub fn clear_buffers(&mut self) {
        self.lines.clear();
    }

    /// WARNING: for GPU safety it's important that Lines are created with correct size (new_gpu_backed fn)
    pub fn push_segments(&mut self, ctx: &mut Context, lines: Lines) {
        self.lines.extend(&lines);
        self.bindings.vertex_buffers[1].update(ctx, &self.lines.0[..]);
    }

    pub fn draw(&mut self, ctx: &mut Context, camera: &Camera) {
        let (width, height) = ctx.screen_size();
        let mvp = camera.get_mvp(height / width);

        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);
        ctx.apply_uniforms(&hex_shader::Uniforms { mvp });
        ctx.draw(0, RECT_INDICES.len() as i32, self.lines.0.len() as i32);
    }
}

mod hex_shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"#version 100
    precision lowp float;
    attribute vec2 pos;
    attribute float segment_type;
    attribute vec2 inst_pos;
    attribute float thickness;
    attribute vec2 dir;
    attribute vec3 color0;

    varying vec2 local_position;
    varying vec2 projected_position;
    varying vec2 ip;
    varying float th;
    varying vec4 color;
    // segment type. Have to pass as float, but it is just enum
    varying float st;
    varying vec2 dr;

    uniform mat4 mvp;
    void main() {
        vec2 n = vec2(-dir.y, dir.x) / length(dir);
        vec2 apos = pos.y * dir + pos.x * n * thickness;
        vec4 new_pos = vec4(apos + inst_pos, 0.0, 1.0);
        vec4 res_pos = mvp * new_pos;
        gl_Position = res_pos;

        st = segment_type;
        local_position = pos;
        projected_position = vec2(new_pos.x, new_pos.y);
        ip = inst_pos;
        dr = dir;
        th = thickness;
        color = vec4(color0, 0.5);
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    precision lowp float;
    varying vec2 local_position;
    varying vec2 projected_position;
    varying vec2 ip;
    varying float th;
    varying vec4 color;
    varying float st;
    varying vec2 dr;

    uniform mat4 mvp;
    const lowp float aaborder = 0.00445;

    float line_segment(in vec2 p, in vec2 a, in vec2 b) {
        vec2 ba = b - a;
        vec2 pa = p - a;
        float h = clamp(dot(pa, ba) / dot(ba, ba), 0., 1.);
        return length(pa - h * ba);
    }

    void main() {
        vec2 a = ip - dr  / 2.;
        vec2 b = ip + dr / 2.;
        float d = line_segment(projected_position, a, b) - th;
        float scaled_border = aaborder / mvp[1][1];
        float edge1 = -scaled_border;
        float edge2 = 0.;

        if (d < 0.) {
            float smooth = 1.;
            if (abs(st - 1.) < 0.01 && local_position.y < -0.5) { // in SDF space
                discard;
            } else if (abs(st - 2.) < 0.01 && local_position.y > 0.5) {
                discard;
            }
            if (d > edge1) {
                smooth = 1. - smoothstep(edge1, edge2, d) + st - st;
            }
            vec4 color = color;
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
