mod camera;
pub mod comms;

use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread, f32::consts::PI,
};

pub use bevy::prelude::Vec3;
use bevy::{
    pbr::{wireframe::{Wireframe, WireframeConfig, WireframePlugin}, NotShadowCaster},
    prelude::*,
    render::{
        settings::{WgpuFeatures, WgpuSettings},
        view::{NoFrustumCulling, ViewDepthTexture},
        RenderPlugin,
    },
};
pub use comms::*;
use ndarray::Axis;
use smooth_bevy_cameras::{
    controllers::{
        orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
        unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin},
        // fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    },
    LookTransform, LookTransformPlugin,
};

pub fn run_rust() {
    let (msender, mreceiver) = channel::<Message>();
    let (rsender, rreceiver) = channel::<Response>();

    thread::spawn(|| listen(msender, rreceiver));

    App::new()
        .add_startup_system(startup)
        .add_system(bevy_listen)
        .add_system(plane_transform)
        .insert_non_send_resource(mreceiver)
        .insert_non_send_resource(rsender)
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            wgpu_settings: WgpuSettings {
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..default()
            },
        }))
        .add_plugin(WireframePlugin)
        .add_plugin(LookTransformPlugin)
        .add_plugin(UnrealCameraPlugin::default())
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(camera::MyCameraPlugin::default())
        // .insert_resource(ClearColor(Color::rgba(0.4, 0.4, 0.4, 0.)))
        .run();
}

#[derive(Component)]
struct TheMesh;

#[derive(Component)]
struct Flashlight;

#[derive(Component)]
struct ViewPlane;

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    wireframe_config.global = false;
    // commands
    //     .spawn(Camera3dBundle::default())
    //     .insert(UnrealCameraBundle::new(
    //         UnrealCameraController::default(),
    //         Vec3::new(-1., -1., -1.),
    //         Vec3::new(0., 0., 0.),
    //         Vec3::Y,
    //     ));

    let mat = materials.add(StandardMaterial{
        double_sided: true,
        cull_mode: None,
        emissive: Color::ORANGE_RED,
        base_color: Color::ORANGE_RED.with_a(0.1),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let plane = PbrBundle{
        mesh: meshes.add(Mesh::from(shape::Plane::from_size(1.))),
        material: mat,
        transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::X, PI/2.)),
        ..default()
    };
    commands
        .spawn(Camera3dBundle::default())
        .insert(camera::MyCameraBundle::new(
            camera::MyCameraController::default(),
            Vec3::new(-1., -1., -1.),
            Vec3::new(0., 0., 0.),
            Vec3::Y,
        ))
        .insert(PointLightBundle {
            point_light: PointLight {
                intensity: 100000.0,
                shadows_enabled: true,
                range: 100.,
                ..default()
            },
            ..default()
        })
        .with_children(|parent|{
            parent.spawn(plane).insert(ViewPlane).insert(NotShadowCaster);
        });
    // commands.spawn(plane);

    // commands
    //     .spawn(Camera3dBundle::default())
    //     .insert(OrbitCameraBundle::new(
    //         OrbitCameraController::default(),
    //         Vec3::new(-1., -1., -1.),
    //         Vec3::new(
    //             0., 0., 0.,
    //         ),
    //         Vec3::Y,
    //     ));

    // commands.spawn(DirectionalLightBundle {
    //     directional_light: DirectionalLight {
    //         illuminance: 10000.,
    //         shadows_enabled: true,
    //         ..Default::default()
    //     },
    //     transform: Transform::from_xyz(0., 0., 1.),
    //     ..default()
    // });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 100000.0,
            shadows_enabled: true,
            range: 100.,
            ..default()
        },
        transform: Transform::from_xyz(-4.0, 8.0, 4.0),
        ..default()
    });


    

    // commands
    //     .spawn(PointLightBundle {
    //         point_light: PointLight {
    //             intensity: 100000.0,
    //             shadows_enabled: true,
    //             range: 100.,
    //             ..default()
    //         },
    //         transform: Transform::from_xyz(-4.0, 8.0, 4.0),
    //         ..default()
    //     })
    //     .insert(Flashlight);

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::new(0., 0., 0.)]);
    let handle = meshes.add(mesh);

    let mut mat: StandardMaterial = Color::rgb(0.3, 0.5, 0.3).into();
    mat.cull_mode = None;
    mat.double_sided = true;
    commands
        .spawn(TheMesh)
        .insert(NoFrustumCulling)
        .insert(PbrBundle {
            mesh: handle.clone(),
            material: materials.add(mat),
            ..default()
        });
        // .insert(Wireframe);
}

fn plane_transform(
    look_transform: Query<&LookTransform, With<Camera>>,
    mut plane: Query<&mut Transform, With<ViewPlane>>
){
    let radius = look_transform.single().radius();
    let mut plane = plane.single_mut();
    plane.translation.z = -radius;
    plane.scale = Vec3::splat(radius);
}

fn bevy_listen(
    receiver: NonSend<Receiver<Message>>,
    sender: NonSend<Sender<Response>>,
    query: Query<(&TheMesh, &Handle<Mesh>)>,
    mut camera: Query<(&Camera, &mut LookTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    match receiver.try_recv() {
        Ok(recv) => {
            let (_, handle) = query.single();
            let mesh = meshes.get_mut(handle).unwrap();
            let (_, mut lookat) = camera.single_mut();

            match recv {
                Message::Mesh { verts, faces } => {
                    let full_verts: Vec<Vec3> = faces
                        .as_slice()
                        .unwrap()
                        .iter()
                        .rev()
                        .map(|&index| {
                            let vert = verts.index_axis(Axis(0), index as usize);
                            Vec3::new(vert[0], vert[1], vert[2])
                        })
                        .collect();
                    let com = full_verts.iter().sum::<Vec3>() / full_verts.len() as f32;
                    lookat.target = com;

                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, full_verts);
                    mesh.compute_flat_normals();
                    // if let Some(aabb) = mesh.compute_aabb(){
                    //     commands.entity(entity).insert(aabb);
                    // }
                }
                Message::SetView(View { position, look_at }) => {
                    lookat.target = look_at;
                    lookat.eye = position;
                }
                Message::RequestView => {
                    let position = lookat.eye;
                    let look_at = lookat.target;
                    let response = Response::GetView(View { position, look_at });
                    sender.send(response).unwrap()
                }
            }
        }
        Err(_) => (),
    }
}
