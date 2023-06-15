use std::{sync::{Arc, mpsc::{channel, Receiver, Sender, TryRecvError}}, thread};

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
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

fn listen(sender: Sender<Message>){
    let listener = TcpListener::bind("localhost:6142").unwrap();
    loop{
        match listener.accept(){
            Ok((mut stream, _addr)) => {
                loop{
                    let recv = Message::receive(&mut stream);
                    match recv.unwrap(){
                        Some(message) => sender.send(message).unwrap(),
                        None => break
                    }
                }
                // handle_client(stream, &sender)
            },
            Err(e) => eprintln!("errored"),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Message{
    Mesh{
        verts: Array2<f32>,
        faces: Array2<i32>,
    }
}

impl Message{
    pub fn send(&self, stream: &mut TcpStream) -> anyhow::Result<()>{
        let bytes = bincode::serialize(self)?;
        let len = usize::to_le_bytes(bytes.len());
        stream.write_all(&len)?;
        stream.write_all(&bytes)?;

        Ok(())
    }

    pub fn receive(stream: &mut TcpStream) -> anyhow::Result<Option<Self>>{
        let mut len = [0; std::mem::size_of::<usize>()];
        let n_read = stream.read(&mut len)?;
        if n_read == 0{
            return Ok(None)
        }
        let len = usize::from_le_bytes(len);
        let mut message = vec![0; len];
        stream.read(&mut message)?;
        let message: Self = bincode::deserialize(&message)?;
        Ok(Some(message))
    }
}

pub fn run_rust() {

    let (sender, receiver): (Sender<Message>, Receiver<Message>) = channel();
    

    thread::spawn(||{
        listen(sender)
    });
    
    App::new()
        .add_startup_system(startup)
        .add_system(bevy_listen)
        .insert_non_send_resource(receiver)
        .add_plugins(DefaultPlugins)
        .add_plugin(LookTransformPlugin)
        .add_plugin(OrbitCameraPlugin::default())
        .add_plugin(UnrealCameraPlugin::default())
        .add_plugin(FpsCameraPlugin::default())
        .insert_resource(ClearColor(Color::rgba(0.4, 0.4, 0.4, 0.)))
        .run();
}

#[derive(Component)]
struct TheMesh;

fn startup(
    mut commands: Commands,
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
    // let full_verts: Vec<Vec3> = faces.0.iter().map(|&index| {
    //     let vert = verts.0.index_axis(Axis(0), index as usize);
    //     Vec3::new(vert[0], vert[1], vert[2])
    // }).collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![Vec3::new(0., 0., 0.)]);
    let handle = meshes.add(mesh);

    let mut mat: StandardMaterial = Color::rgb(0.8, 0.7, 0.6).into();
    mat.cull_mode = None;
    commands.spawn((TheMesh, PbrBundle {
        mesh: handle.clone(),
        material: materials.add(mat),
        ..default()
    }));
}

fn bevy_listen(
    nonsend: NonSend<Receiver<Message>>,
    query: Query<(&TheMesh, &Handle<Mesh>)>,
    mut meshes: ResMut<Assets<Mesh>>,
){
    
    match nonsend.try_recv(){
        Ok(recv) => {
            // dbg!(&recv);
            let (_, handle) = query.single();
            let mesh = meshes.get_mut(handle).unwrap();
            
            match recv{
                Message::Mesh { verts, faces } => {
                    let full_verts: Vec<Vec3> = faces.iter().map(|&index| {
                        let vert = verts.index_axis(Axis(0), index as usize);
                        Vec3::new(vert[0], vert[1], vert[2])
                    }).collect();
            
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, full_verts);
                },
            }
            
        },
        Err(_) => (),
    }
}


