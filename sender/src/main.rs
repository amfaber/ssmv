use std::net::TcpStream;

// use std::io::Write;

use ndarray::{array, Array1, Array2};
use super_simple_mesh_viewer::{Communication, Message};

fn send() {
    let verts = array![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.],];

    let faces = array![[0, 1, 2],];

    let message = Message::Mesh { verts, faces };

    match TcpStream::connect("localhost:6142") {
        Ok(mut stream) => {
            message.send(&mut stream).unwrap();
        }
        Err(e) => panic!("failed on main thread, {:?}", e),
    }
}

fn serde_array() {
    // let verts: Array2<f32> = array![[4f32, 5., 1.], [0., 1., 7.]];
    // let faces: Array2<i32> = array![[0, 1, 2]];
    // let message = Message::Mesh { verts, faces  };
    // let ser = bincode::serialize(&message).unwrap();
    // let des = bincode::deserialize::<Message>(&ser).unwrap();
    // dbg!(des);

    // let ser = serde_json::to_string_pretty(&message).unwrap();
    // println!("{}", &ser);
    // let des = serde_json::from_str::<Message>(&ser).unwrap();
    // dbg!(des);
    // let ser = serde_json::serialize(&message).unwrap();
    // let des = serde_json::deserialize::<Message>(&ser).unwrap();
}

fn main() {
    serde_array();
    // send()
}
