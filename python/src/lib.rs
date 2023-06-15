use std::{net::{TcpStream, SocketAddr, ToSocketAddrs}, process::Stdio};

use super_simple_mesh_viewer::{run_rust, Message};
use numpy::{PyReadonlyArray2, array, PyArray};
use pyo3::prelude::*;



#[pyfunction]
fn run() {
    run_rust();
}

#[pyclass]
pub struct Connection{
    tcp: TcpStream,
}

#[pymethods]
impl Connection{
    #[new]
    fn new() -> PyResult<Self>{
        let addr = "localhost:6142".to_socket_addrs().unwrap().next().unwrap();
        let timeout = std::time::Duration::from_millis(50);
        let boot = std::time::Duration::from_secs(5);

        // let verts = verts.as_array().to_owned();
        // let faces = faces.as_array().to_owned();
    
        // let message = Message::Mesh { verts, faces };
    
        let tcp = match TcpStream::connect_timeout(&addr, timeout){
            Ok(stream) => {
                stream
            },
            Err(_) => {
                std::process::Command::new(r"python")
                    .arg("-c")
                    .arg("import ssmv; ssmv.run()")
                    .stdout(Stdio::null())
                    .spawn().unwrap();
                TcpStream::connect_timeout(&addr, boot).unwrap()
                // message.send(&mut stream).unwrap();
            },
        };
        
        Ok(Self{ tcp })
    }

    fn send(&mut self, verts: PyReadonlyArray2<f32>, faces: PyReadonlyArray2<i32>){
        let verts = verts.as_array().to_owned();
        let faces = faces.as_array().to_owned();
    
        let message = Message::Mesh { verts, faces };
        message.send(&mut self.tcp).unwrap();
    }
    
    fn test<'py>(&mut self, py: Python<'py>) -> PyResult<()>{
        let verts = array![
            [0., 0., 0.],
            [1., 0., 0.],
            [0., 1., 0.],
        ];

        let faces = array![
            [0, 1, 2],
        ];

        let verts = PyArray::from_owned_array(py, verts);
        let faces = PyArray::from_owned_array(py, faces);
        // let mut connection = Connection::new().unwrap();
        self.send(verts.readonly(), faces.readonly());
        // send(verts.readonly(), faces.readonly());

        Ok(())
    }

}

// #[pyfunction]
// fn test<'py>(py: Python<'py>) -> PyResult<()>{
    
//     // send(verts.readonly(), faces.readonly());

//     Ok(())
// }

#[pymodule]
fn ssmv(_py: Python<'_>, m: &PyModule) -> PyResult<()>{
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_class::<Connection>()?;
    Ok(()) 
}
