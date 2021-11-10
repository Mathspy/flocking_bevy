use bevy::{
    app::App,
    asset::Assets,
    ecs::{
        component::Component,
        system::{Commands, Query, Res, ResMut},
    },
    math::{Quat, Vec2},
    prelude::{Handle, Transform},
    render::{
        entity::{MeshBundle, OrthographicCameraBundle},
        mesh::{Indices, Mesh},
        pipeline::{PipelineDescriptor, PrimitiveTopology, RenderPipeline, RenderPipelines},
        shader::{Shader, ShaderStage, ShaderStages},
    },
    window::Windows,
    DefaultPlugins,
};
use rand::{thread_rng, Rng};
use std::num::FpCategory;

#[derive(Component)]
struct Velocity {
    vector: Vec2,
    max: f32,
}
#[derive(Component)]
struct Force {
    vector: Vec2,
    max: f32,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(start)
        .add_system(chase_mouse)
        .add_system(apply_force)
        .add_system(update_boids)
        .run();
}

fn create_boid_mesh_bundle(
    pipeline_handle: Handle<PipelineDescriptor>,
    mesh: Handle<Mesh>,
    coordinates: Vec2,
) -> MeshBundle {
    MeshBundle {
        mesh,
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            pipeline_handle,
        )]),
        transform: Transform::from_xyz(coordinates.x, coordinates.y, 0.0),
        ..Default::default()
    }
}

fn start(
    mut commands: Commands,
    // We will add a new Mesh for the star being created
    mut meshes: ResMut<Assets<Mesh>>,
    // A pipeline will be added with custom shaders
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    // Access to add new shaders
    mut shaders: ResMut<Assets<Shader>>,
    windows: Res<Windows>,
) {
    // We first create a pipeline, which is the sequence of steps that are
    // needed to get to pixels on the screen starting from a description of the
    // geometries in the scene. Pipelines have fixed steps, which sometimes can
    // be turned off (for instance, depth and stencil tests) and programmable
    // steps, the vertex and fragment shaders, that we can customize writing
    // shader programs.

    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        // Vertex shaders are run once for every vertex in the mesh.
        // Each vertex can have attributes associated to it (e.g. position,
        // color, texture mapping). The output of a shader is per-vertex.
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        // Fragment shaders are run for each pixel belonging to a triangle on
        // the screen. Their output is per-pixel.
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    let mut rng = thread_rng();

    // A mesh can be reused! We need a mesh per shape/color though!
    // So for example a red triangle would need a different mesh, but most other triangles can
    // actually reuse this mesh with some transform stretching if we wanted
    let mut triangle = Mesh::new(PrimitiveTopology::TriangleList);
    triangle.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[-10.0, -5.0, 0.0], [0.0, 0.0, 0.0], [-10.0, 5.0, 0.0]],
    );
    triangle.set_attribute(
        Mesh::ATTRIBUTE_COLOR,
        vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]],
    );
    triangle.set_indices(Some(Indices::U32(vec![0, 1, 2])));
    let mesh_handle = meshes.add(triangle);

    if let Some(window) = windows.as_ref().get_primary() {
        (0..100).for_each(|_| {
            let width = window.width();
            let x = rng.gen_range(-width / 2.0..width / 2.0);
            let height = window.height();
            let y = rng.gen_range(-height / 2.0..height / 2.0);

            let triangle = create_boid_mesh_bundle(
                pipeline_handle.clone(),
                mesh_handle.clone(),
                Vec2::new(x, y),
            );

            commands
                .spawn_bundle(triangle)
                .insert(Velocity {
                    vector: Vec2::new(0.0, 0.0),
                    max: 1.0,
                })
                .insert(Force {
                    vector: Vec2::new(0.0, 0.0),
                    max: 0.25,
                });
        });
    }

    commands
        // And use an orthographic projection
        .spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn update_boids(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        let transform = transform.as_mut();
        // new position = current position + velocity
        transform.translation += velocity.vector.extend(0.0);
        // If there's no velocity then setting the rotation causes the object not to render due
        // to angle_between's calculation containing a division by Sqrt(Mag(A)^2 * Mag(B)^2)
        // which in case of B being 0 would be 0 so division by 0 would result in a NaN
        // Attempting to render with NaN as angle just results in no rendering happening
        //
        // One could argue that setting the angle to 0 or something like that should be done if
        // the object has no velocity but then any object with no velocity would immediately face
        // the right side, which makes no sense.
        // In real life objects preserve their facing direction even after losing all their
        // velocity, this does just that by refusing to set the rotation when there is no velocity
        // thus preserving the previous rotation!
        if velocity.vector.length().classify() != FpCategory::Zero {
            // angle
            transform.rotation =
                Quat::from_rotation_z(Vec2::new(1.0, 0.0).angle_between(velocity.vector));
        }
    }
}

fn chase_mouse(windows: Res<Windows>, mut query: Query<(&mut Force, &Velocity, &Transform)>) {
    if let Some(window) = windows.as_ref().get_primary() {
        if let Some(cursor) = window.cursor_position() {
            let real_cursor_position = cursor - Vec2::new(window.width(), window.height()) / 2.0;
            for (mut force, velocity, Transform { translation, .. }) in query.iter_mut() {
                let force = force.as_mut();
                // target - position
                let desired_velocity = real_cursor_position - translation.truncate();

                // steering force = desired velocity - current velocity
                force.vector =
                    Vec2::clamp_length_max(desired_velocity - velocity.vector, force.max);
            }
        }
    }
}

fn apply_force(mut query: Query<(&mut Velocity, &mut Force)>) {
    for (mut velocity, mut force) in query.iter_mut() {
        let force = force.as_mut();
        let velocity = velocity.as_mut();
        // velocity = current velocity + acceleration; acceleration = force if mass = 1
        velocity.vector = Vec2::clamp_length_max(velocity.vector + force.vector, velocity.max);
        // Once a force is applied it is removed
        force.vector = Vec2::ZERO;
    }
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
