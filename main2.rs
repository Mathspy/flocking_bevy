use bevy::{
    app::App,
    asset::Assets,
    ecs::system::{Commands, ResMut},
    math::{Vec3, Vec4},
    render::{
        color::Color,
        entity::{MeshBundle, OrthographicCameraBundle},
        mesh::{Indices, Mesh},
        pipeline::{PipelineDescriptor, PrimitiveTopology, RenderPipeline, RenderPipelines},
        shader::{Shader, ShaderStage, ShaderStages},
    },
    transform::components::Transform,
    DefaultPlugins,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(start)
        .run();
}

const CIRCLE_VERTICES: u32 = 50;

fn create_circle_mesh() -> Mesh {
    let color = Vec4::from(Color::rgb_u8(255, 127, 80).as_rgba_linear())
        .truncate()
        .to_array();

    let mut circle = Mesh::new(PrimitiveTopology::TriangleList);

    let (positions, colors) = std::iter::once(([0.0, 0.0, 0.0], color))
        .chain((0..CIRCLE_VERTICES).map(|i| {
            let a = i as f32 * std::f32::consts::TAU / (CIRCLE_VERTICES as f32);

            ([a.cos(), a.sin(), 0.0], color)
        }))
        .unzip::<_, _, Vec<[f32; 3]>, Vec<[f32; 3]>>();
    circle.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    // circle.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    circle.set_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    // circle.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    let indices = std::iter::once([0, CIRCLE_VERTICES, 1])
        .chain((2..=CIRCLE_VERTICES).map(|i| [0, i - 1, i]))
        .flatten()
        .collect();
    circle.set_indices(Some(Indices::U32(indices)));

    circle
}

fn start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    let mesh = meshes.add(create_circle_mesh());

    commands.spawn_bundle(MeshBundle {
        mesh,
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            pipeline_handle,
        )]),
        transform: Transform::from_scale(Vec3::new(100.0, 100.0, 1.0)),
        ..Default::default()
    });

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

const VERTEX_SHADER: &str = r"
#version 450
layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Color;
layout(location = 1) out vec3 v_Color;
layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(set = 1, binding = 0) uniform Transform {
    mat4 Model;
};
void main() {
    v_Color = Vertex_Color;
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
";

const FRAGMENT_SHADER: &str = r"
#version 450
layout(location = 1) in vec3 v_Color;
layout(location = 0) out vec4 o_Target;
void main() {
    o_Target = vec4(v_Color, 1.0);
}
";
