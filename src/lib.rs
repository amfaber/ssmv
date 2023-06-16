pub mod comms;

use std::{sync::mpsc::{channel, Receiver, Sender}, thread};

use bevy::prelude::*;
use ndarray::Axis;
use smooth_bevy_cameras::{
    controllers::{
        // orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
        unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin},
        // fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    },
    LookTransform, LookTransformPlugin,
};
pub use comms::*;
pub use bevy::prelude::Vec3;


pub fn run_rust() {

    let (msender, mreceiver) = channel::<Message>();
    let (rsender, rreceiver) = channel::<Response>();
    

    thread::spawn(||{
        listen(msender, rreceiver)
    });
    
    App::new()
        .add_startup_system(startup)
        .add_system(bevy_listen)
        .insert_non_send_resource(mreceiver)
        .insert_non_send_resource(rsender)
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(UnrealCameraPlugin::default())
        // .insert_resource(ClearColor(Color::rgba(0.4, 0.4, 0.4, 0.)))
        .run();
}

#[derive(Component)]
struct TheMesh;

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    
    commands
        .spawn(Camera3dBundle::default())
        .insert(UnrealCameraBundle::new(
            UnrealCameraController::default(),
            Vec3::new(-1., -1., -1.),
            Vec3::new(
                0., 0., 0.,
            ),
            Vec3::Y,
        ));

    
    commands
        .spawn(DirectionalLightBundle{
            directional_light: DirectionalLight{
                illuminance: 10000.,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 0., 1.),
            ..default()
        });


    commands
        .insert_resource(AmbientLight{
            color: Color::WHITE,
            brightness: 0.2,
        });
    // light
    // PointLight::default()
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 3000.0,
    //         shadows_enabled: true,
    //         range: 100.,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(-4.0, 8.0, 4.0),
    //     ..default()
    // });
    
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::new(0., 0., 0.)]);
    let handle = meshes.add(mesh);

    let mut mat: StandardMaterial = Color::rgb(0.3, 0.5, 0.3).into();
    mat.cull_mode = None;
    mat.double_sided = true;
    commands.spawn((TheMesh, PbrBundle {
        mesh: handle.clone(),
        material: materials.add(mat),
        ..default()
    }));
}

fn bevy_listen(
    receiver: NonSend<Receiver<Message>>,
    sender: NonSend<Sender<Response>>,
    query: Query<(&TheMesh, &Handle<Mesh>)>,
    mut camera: Query<(&Camera, &mut LookTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    
){
    match receiver.try_recv(){
        Ok(recv) => {
            let (_, handle) = query.single();
            let mesh = meshes.get_mut(handle).unwrap();
            let (_, mut lookat) = camera.single_mut();
            
            match recv{
                Message::Mesh { verts, faces } => {
                    let full_verts: Vec<Vec3> = faces.as_slice().unwrap().iter().rev().map(|&index| {
                        let vert = verts.index_axis(Axis(0), index as usize);
                        Vec3::new(vert[0], vert[1], vert[2])
                    }).collect();
                    let com = full_verts.iter().sum::<Vec3>() / full_verts.len() as f32;
                    lookat.target = com;
                    
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, full_verts);
                    mesh.compute_flat_normals();
                },
                Message::SetView(View{ position, look_at }) => {
                    lookat.target = look_at;
                    lookat.eye = position;
                },
                Message::RequestView => {
                    let position = lookat.eye;
                    let look_at = lookat.target;
                    let response = Response::GetView(View{
                        position,
                        look_at,
                    });
                    sender.send(response).unwrap()
                },
            }
            
        },
        Err(_) => (),
    }
}


