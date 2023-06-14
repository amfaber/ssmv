use bevy::prelude::*;
use ndarray::{Array2, Axis};
use smooth_bevy_cameras::{
    controllers::{
        orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
        unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin},
        fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    },
    LookTransform, LookTransformBundle, LookTransformPlugin, Smoother,
};

#[derive(Resource)]
struct Vertices(Array2<f32>);
#[derive(Resource)]
struct Faces(Array2<i32>);

pub fn run_rust(verts: Array2<f32>, faces: Array2<i32>) {
    App::new()
        .add_startup_system(startup)
        .insert_resource(Vertices(verts))
        .insert_resource(Faces(faces))
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(UnrealCameraPlugin::default())
        .add_plugin(FpsCameraPlugin::default())
        .insert_resource(ClearColor(Color::rgba(0.4, 0.4, 0.4, 0.)))
        .run();
}

fn startup(
    mut commands: Commands,
    faces: Res<Faces>,
    verts: Res<Vertices>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    
    // commands
    //     .spawn(Camera3dBundle::default())
    //     .insert(UnrealCameraBundle::new(
    //         UnrealCameraController::default(),
    //         Vec3::new(-2., 5., 5.),
    //         Vec3::new(
    //             0., 0., 0.,
    //         ),
    //         Vec3::Y,
    //     ));

    commands
        .spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            Vec3::new(-2., 5., 5.),
            Vec3::new(
                0., 0., 0.,
            ),
            Vec3::Y,
        ));

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    let full_verts: Vec<Vec3> = faces.0.iter().map(|&index| {
        let vert = verts.0.index_axis(Axis(0), index as usize);
        Vec3::new(vert[0], vert[1], vert[2])
    }).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, full_verts);
    let handle = meshes.add(mesh);

    let mut mat: StandardMaterial = Color::rgb(0.8, 0.7, 0.6).into();
    mat.cull_mode = None;
    commands.spawn(PbrBundle {
        mesh: handle.clone(),
        material: materials.add(mat),
        ..default()
    });
}


