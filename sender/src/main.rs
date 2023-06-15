use std::net::TcpStream;

// use std::io::Write;

use ndarray::array;
use super_simple_mesh_viewer::Message;

fn send(){

    let verts = array![
        [0., 0., 0.],
        [1., 0., 0.],
        [0., 1., 0.],
    ];

    let faces = array![
        [0, 1, 2],
    ];

    let message = Message::Mesh{
        verts,
        faces,
    };

    match TcpStream::connect("localhost:6142"){
        Ok(mut stream) => {
            message.send(&mut stream).unwrap();
        },
        Err(e) => panic!("failed on main thread, {:?}", e),
    }
}

fn main() {
    send()
}
